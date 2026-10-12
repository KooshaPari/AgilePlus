"""Execute the checked-in gateway against disposable loopback upstreams.

Requires CADDY_BIN (or caddy on PATH). This proves local proxy behavior,
not production DNS, certificates, tailnet ACLs or application acceptance.
"""
import base64
import http.client
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]
ORIGIN = "https://agileplus.pheno.studio"


class Upstream(BaseHTTPRequestHandler):
    requests = []

    def handle_request(self):
        self.requests.append((self.command, self.path, dict(self.headers)))
        self.send_response(200)
        self.send_header("Set-Cookie", "upstream=untrusted")
        self.end_headers()
        self.wfile.write(b"fixture")

    do_GET = do_POST = do_PUT = do_PATCH = do_DELETE = handle_request

    def log_message(self, *_):
        pass


class OperatorGateway(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        caddy = os.environ.get("CADDY_BIN") or shutil.which("caddy")
        if not caddy:
            raise RuntimeError("Real Caddy is required; gateway evidence cannot be skipped")
        cls.temp = tempfile.TemporaryDirectory()
        cls.upstream = ThreadingHTTPServer(("127.0.0.1", 0), Upstream)
        cls.thread = threading.Thread(target=cls.upstream.serve_forever, daemon=True)
        cls.thread.start()
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            cls.port = sock.getsockname()[1]
        upstream = f"127.0.0.1:{cls.upstream.server_port}"
        source = (ROOT / "deploy/selfhost/Caddy.operator-alpha.caddy").read_text()
        source = source.replace("agileplus.pheno.studio {", f"http://127.0.0.1:{cls.port} {{")
        source = source.replace("reverse_proxy 127.0.0.1:3000", f"reverse_proxy {upstream}")
        source = source.replace("reverse_proxy https://{$AGILEPLUS_FRONTEND_UPSTREAM_HOST}", f"reverse_proxy http://{upstream}")
        config = Path(cls.temp.name) / "Caddyfile"
        config.write_text("{\n admin off\n auto_https off\n}\n" + source)
        env = dict(os.environ)
        env.update(AGILEPLUS_OPERATOR_USERNAME="smoke", AGILEPLUS_API_KEY="backend-fixture-key",
                   AGILEPLUS_FRONTEND_UPSTREAM_HOST="fixture.vercel.app")
        env["AGILEPLUS_OPERATOR_PASSWORD_HASH"] = subprocess.check_output(
            [caddy, "hash-password", "--plaintext", "fixture-password"], text=True).strip()
        cls.log = open(Path(cls.temp.name) / "caddy.log", "w+")
        cls.process = subprocess.Popen([caddy, "run", "--config", str(config), "--adapter", "caddyfile"],
                                       env=env, stdout=cls.log, stderr=cls.log)
        for _ in range(100):
            if cls.process.poll() is not None:
                cls.log.seek(0)
                raise RuntimeError(cls.log.read())
            try:
                with socket.create_connection(("127.0.0.1", cls.port), timeout=.1):
                    return
            except OSError:
                time.sleep(.05)
        raise RuntimeError("Caddy did not start")

    @classmethod
    def tearDownClass(cls):
        cls.process.terminate()
        cls.process.wait(timeout=5)
        cls.upstream.shutdown()
        cls.upstream.server_close()
        cls.log.close()
        cls.temp.cleanup()

    def request(self, method="GET", origin=None, auth=True, path="/api/v1/features"):
        headers = {"Cookie": "private=secret", "X-API-Key": "browser-spoof"}
        if auth:
            headers["Authorization"] = "Basic " + base64.b64encode(b"smoke:fixture-password").decode()
        if origin is not None:
            headers["Origin"] = origin
        conn = http.client.HTTPConnection("127.0.0.1", self.port, timeout=3)
        try:
            conn.request(method, path, headers=headers)
            response = conn.getresponse()
            body = response.read()
            return response.status, dict(response.getheaders()), body
        finally:
            conn.close()

    def test_unauthenticated_never_reaches_backend(self):
        before = len(Upstream.requests)
        self.assertEqual(self.request(auth=False)[0], 401)
        self.assertEqual(len(Upstream.requests), before)

    def test_cross_site_and_missing_origins_never_mutate(self):
        for method in ["POST", "PUT", "PATCH", "DELETE"]:
            for origin in [None, "null", "https://evil.example", ORIGIN + ".evil.example", ORIGIN + ":444"]:
                with self.subTest(method=method, origin=origin):
                    before = len(Upstream.requests)
                    self.assertEqual(self.request(method, origin)[0], 403)
                    self.assertEqual(len(Upstream.requests), before)

    def test_same_origin_mutations_inject_only_host_key(self):
        for method in ["POST", "PUT", "PATCH", "DELETE"]:
            self.assertEqual(self.request(method, ORIGIN)[0], 200)
            headers = {k.lower(): v for k, v in Upstream.requests[-1][2].items()}
            self.assertEqual(headers["x-api-key"], "backend-fixture-key")
            self.assertNotIn("authorization", headers)

    def test_frontend_has_no_operator_credentials_or_cookies(self):
        status, headers, body = self.request(path="/")
        self.assertEqual(status, 200)
        upstream = {k.lower(): v for k, v in Upstream.requests[-1][2].items()}
        self.assertNotIn("authorization", upstream)
        self.assertNotIn("cookie", upstream)
        self.assertNotIn("x-api-key", upstream)
        self.assertNotIn("set-cookie", {k.lower(): v for k, v in headers.items()})
        self.assertNotIn(b"backend-fixture-key", body)


if __name__ == "__main__":
    unittest.main()
