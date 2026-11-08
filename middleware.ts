// Vercel Routing Middleware: require a valid Cloudflare Access JWT on every request.
// The direct <project>.vercel.app production alias is publicly reachable even with
// Vercel Standard Protection (production is exempt), so the edge rejects any
// request that did not arrive through an Access-gated hostname with 401.
//
// Verifies Cf-Access-Jwt-Assertion against the Access team JWKS
// (https://floral-moon-c604.cloudflareaccess.com/cdn-cgi/access/certs),
// the team issuer, expiry, and a known Access application AUD.

const TEAM_DOMAIN = 'floral-moon-c604.cloudflareaccess.com';
const CERTS_URL = `https://${TEAM_DOMAIN}/cdn-cgi/access/certs`;
const ISSUER = `https://${TEAM_DOMAIN}`;

// Access application AUDs accepted by this deployment (all "Koosha only" policy):
//   pheno-studio-wildcard (*.pheno.studio), pheno-agileplus, pheno-tracera,
//   pheno-shop-wildcard (*.pheno.shop), pheno-agileplus-space (agileplus.phenotype.space)
const TRUSTED_AUDS = [
  'cd5a353030d8379b9322508233a0dbeaaa6896103c6237fa54cdb907e2b5852a',
  '019aa5a73cbde4c84fcac461f8ff1c7f0d0489e65df767d7a4f91759855f8a76',
  '1502c30a5519bf4a7dbd7ad0148e5d2b9d951aa8371d64ff4858bdc468396b27',
  'ac34527585f7e7c796b8fade8ea011f998ed019bc90bc227949324181224efc3',
  'a47099890d858f9ac98e4388b9c3ec007d78b6404c40348717f111018065e6a8',
];

let keysPromise: Promise<Map<string, CryptoKey>> | null = null;
let keysFetchedAt = 0;

async function getKeys(): Promise<Map<string, CryptoKey>> {
  if (!keysPromise || Date.now() - keysFetchedAt > 3600_000) {
    keysFetchedAt = Date.now();
    keysPromise = (async () => {
      const res = await fetch(CERTS_URL);
      if (!res.ok) throw new Error(`JWKS fetch failed: ${res.status}`);
      const { keys } = (await res.json()) as { keys: Array<JsonWebKey & { kid?: string }> };
      const map = new Map<string, CryptoKey>();
      for (const k of keys) {
        if (k.kty !== 'RSA' || !k.kid) continue;
        map.set(
          k.kid,
          await crypto.subtle.importKey(
            'jwk',
            k,
            { name: 'RSASSA-PKCS1-v1_5', hash: 'SHA-256' },
            false,
            ['verify'],
          ),
        );
      }
      return map;
    })();
    // Attach the rejection handler to the promise chain, not to a copy
    // evaluated after the fact. `keysPromise.catch(...)` registers a handler
    // on the in-flight promise, so a fetch that rejects BEFORE this line
    // runs would otherwise leave the rejected promise cached for the whole
    // hour-long TTL and every later request would fail on it.
    keysPromise = keysPromise.catch((err) => {
      keysPromise = null;
      throw err;
    });
  }
  return keysPromise;
}

function b64url(segment: string): Uint8Array<ArrayBuffer> {
  const b64 = segment.replace(/-/g, '+').replace(/_/g, '/');
  const bin = atob(b64);
  const bytes = new Uint8Array(new ArrayBuffer(bin.length));
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes;
}

async function hasValidAccessJwt(jwt: string): Promise<boolean> {
  const parts = jwt.split('.');
  if (parts.length !== 3) return false;
  let header: { alg?: string; kid?: string };
  let payload: { iss?: string; aud?: string | string[]; exp?: number };
  try {
    header = JSON.parse(new TextDecoder().decode(b64url(parts[0])));
    payload = JSON.parse(new TextDecoder().decode(b64url(parts[1])));
  } catch {
    return false;
  }
  // JSON.parse can yield null or a scalar (e.g. a crafted "null" segment); without
  // this guard the property reads below would throw and surface a 500 instead of 401.
  if (!header || typeof header !== 'object' || !payload || typeof payload !== 'object') {
    return false;
  }
  if (header.alg !== 'RS256' || !header.kid) return false;
  if (payload.iss !== ISSUER) return false;
  const auds = Array.isArray(payload.aud) ? payload.aud : [payload.aud];
  if (!auds.some((a) => a && TRUSTED_AUDS.includes(a))) return false;
  // Allow a small clock-skew leeway. The Access edge re-issues a token per
  // request, so a token minted microseconds before the edge's clock reads can
  // arrive at a node whose clock is slightly behind and be rejected as
  // already-expired. 30s of leeway is well inside Cloudflare Access's own
  // token lifetime and does not extend a token's usable window meaningfully.
  const EXPIRY_LEEWAY_SECONDS = 30;
  if (typeof payload.exp !== 'number' || payload.exp + EXPIRY_LEEWAY_SECONDS < Date.now() / 1000) {
    return false;
  }
  try {
    let keys = await getKeys();
    let key = keys.get(header.kid);
    if (!key && Date.now() - keysFetchedAt > 60_000) {
      // Unknown kid after a key rotation: drop the cached JWKS and refetch once
      // (guarded so a flood of junk kids cannot trigger a fetch storm).
      keysPromise = null;
      keys = await getKeys();
      key = keys.get(header.kid);
    }
    if (!key) return false;
    const signed = new TextEncoder().encode(`${parts[0]}.${parts[1]}`);
    return await crypto.subtle.verify('RSASSA-PKCS1-v1_5', key, b64url(parts[2]), signed);
  } catch {
    return false;
  }
}

function constantTimeEqual(a: string, b: string): boolean {
  // Length is not secret (both are fixed-length host env values), but the
  // CONTENT comparison must not short-circuit on the first differing byte.
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

export default async function middleware(request: Request): Promise<Response | undefined> {
  // Gate production deployments only. The public <project>.vercel.app production
  // alias is exempt from Vercel Authentication, while preview deployments already
  // require Vercel Authentication and cannot carry a Cloudflare Access JWT, so
  // gating previews would make every preview URL unusable.
  if (process.env.VERCEL_ENV !== 'production') return undefined;

  // Path 1: Cloudflare Access. The edge injects cf-access-jwt-assertion after
  // authenticating the operator against the Access application.
  const jwt = request.headers.get('cf-access-jwt-assertion');
  if (jwt && (await hasValidAccessJwt(jwt))) return undefined; // continue to the app

  // Path 2: the private/operator tailnet edge (deploy/selfhost/Caddy.operator-alpha.caddy).
  // That edge authenticates with Tailscale ACL + HTTP Basic and then proxies the
  // frontend to this deployment, stripping Authorization/Cookie/X-API-Key. It sits
  // OUTSIDE Cloudflare Access, so no cf-access-jwt-assertion is ever produced and
  // path 1 can never succeed for it -- without path 2 the operator's own frontend
  // would 401 against its own deployment.
  //
  // The edge attests itself with a shared secret it holds in
  // AGILEPLUS_EDGE_ATTESTATION (never checked in, host-only env, like the other
  // AGILEPLUS_* values the Caddy fragment already documents). The header is
  // stripped from any client-originated request at the edge, and it is compared
  // in constant time.
  const attestation = request.headers.get('x-agileplus-edge-attestation');
  const expected = process.env.AGILEPLUS_EDGE_ATTESTATION;
  if (attestation && expected && constantTimeEqual(attestation, expected)) {
    return undefined; // continue to the app
  }

  return new Response('Unauthorized', {
    status: 401,
    headers: { 'content-type': 'text/plain; charset=utf-8' },
  });
}
