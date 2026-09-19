//! In-process RESP (Redis protocol) server double.
//!
//! The crate's Redis-backed adapters (`RedisCacheStore`, `RateLimiter`,
//! `ProjectionCache`, `CacheHealthChecker`, `CachePool`) cannot be exercised
//! without a server speaking the Redis wire protocol. Tests must not depend on
//! an externally running Dragonfly/Redis instance, so this module implements a
//! minimal RESP2 server on `127.0.0.1:<ephemeral>` that supports exactly the
//! commands those adapters issue, and records what it received.
//!
//! Determinism notes:
//! - The listener binds an OS-assigned port on the loopback interface, so the
//!   double never collides with a real cache server.
//! - No environment variables and no global process state are touched.
//! - Behaviour is fully in-process; there is nothing to time out.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// One command the server received, with its raw arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedCommand {
    /// Uppercased command name, e.g. `SETEX`.
    pub name: String,
    /// Arguments in wire order, decoded lossily as UTF-8.
    pub args: Vec<String>,
}

/// A single command that the server must answer with an error reply.
#[derive(Debug, Clone)]
struct InjectedFailure {
    command: String,
    message: String,
}

#[derive(Debug, Default)]
struct ServerState {
    values: Mutex<HashMap<String, String>>,
    received: Mutex<Vec<ReceivedCommand>>,
    /// TTLs observed on `SETEX`, in call order.
    setex_ttls: Mutex<Vec<(String, i64)>>,
    /// Command that should fail, if any.
    failure: Mutex<Option<InjectedFailure>>,
    /// Payload returned for `PING`; anything other than `PONG` is treated as an
    /// unhealthy server by `RedisConnectionManager::is_valid`.
    ping_reply: Mutex<String>,
    connections_accepted: AtomicU64,
}

/// Options for a `PING` reply that is not a healthy `PONG`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingBehaviour {
    /// Reply `+PONG` (healthy).
    Pong,
    /// Reply `+NOPE`, i.e. a syntactically valid but wrong payload.
    UnexpectedPayload,
}

/// A running RESP2 double. Dropping it stops accepting new connections.
pub struct MockRedisServer {
    state: Arc<ServerState>,
    port: u16,
    _accept_loop: tokio::task::JoinHandle<()>,
}

impl MockRedisServer {
    /// Start a server on an OS-assigned loopback port.
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let port = listener.local_addr().expect("listener local addr").port();

        let state = Arc::new(ServerState {
            ping_reply: Mutex::new("PONG".to_string()),
            ..ServerState::default()
        });

        let accept_state = Arc::clone(&state);
        let accept_loop = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    return;
                };
                accept_state
                    .connections_accepted
                    .fetch_add(1, Ordering::SeqCst);
                let connection_state = Arc::clone(&accept_state);
                tokio::spawn(async move {
                    serve_connection(stream, connection_state).await;
                });
            }
        });

        Self {
            state,
            port,
            _accept_loop: accept_loop,
        }
    }

    /// Loopback port the server is listening on.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Host to hand to `CacheConfig::new`.
    pub fn host(&self) -> String {
        "127.0.0.1".to_string()
    }

    /// `redis://127.0.0.1:<port>` URL for the running server.
    pub fn url(&self) -> String {
        format!("redis://{}:{}", self.host(), self.port)
    }

    /// Number of TCP connections accepted so far.
    pub fn connections_accepted(&self) -> u64 {
        self.state.connections_accepted.load(Ordering::SeqCst)
    }

    /// Commands received so far, in order, across all connections.
    pub fn received(&self) -> Vec<ReceivedCommand> {
        self.state.received.lock().expect("received lock").clone()
    }

    /// Names of the commands received so far, in order.
    pub fn command_names(&self) -> Vec<String> {
        self.received().into_iter().map(|c| c.name).collect()
    }

    /// Count of commands with the given (case-insensitive) name.
    pub fn count_commands(&self, name: &str) -> usize {
        self.command_names()
            .into_iter()
            .filter(|seen| seen.eq_ignore_ascii_case(name))
            .count()
    }

    /// Arguments of the first command with the given name.
    pub fn first_command(&self, name: &str) -> Option<ReceivedCommand> {
        self.received()
            .into_iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// Raw value stored for `key`, if any.
    pub fn stored(&self, key: &str) -> Option<String> {
        self.state
            .values
            .lock()
            .expect("values lock")
            .get(key)
            .cloned()
    }

    /// All keys currently stored.
    pub fn stored_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self
            .state
            .values
            .lock()
            .expect("values lock")
            .keys()
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    /// `(key, ttl)` pairs observed on `SETEX`, in call order.
    pub fn setex_ttls(&self) -> Vec<(String, i64)> {
        self.state.setex_ttls.lock().expect("setex lock").clone()
    }

    /// Pre-populate a key as if it had been written directly to the server.
    pub fn seed(&self, key: &str, value: &str) {
        self.state
            .values
            .lock()
            .expect("values lock")
            .insert(key.to_string(), value.to_string());
    }

    /// Make the next `command` fail with a server error reply.
    pub fn fail_command(&self, command: &str, message: &str) {
        *self.state.failure.lock().expect("failure lock") = Some(InjectedFailure {
            command: command.to_ascii_uppercase(),
            message: message.to_string(),
        });
    }

    /// Stop failing commands.
    pub fn clear_failure(&self) {
        *self.state.failure.lock().expect("failure lock") = None;
    }

    /// Configure the `PING` reply.
    pub fn set_ping_behaviour(&self, behaviour: PingBehaviour) {
        let reply = match behaviour {
            PingBehaviour::Pong => "PONG",
            PingBehaviour::UnexpectedPayload => "NOPE",
        };
        *self.state.ping_reply.lock().expect("ping lock") = reply.to_string();
    }
}

async fn serve_connection(mut stream: TcpStream, state: Arc<ServerState>) {
    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk = [0_u8; 4096];

    loop {
        let read = match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };
        buffer.extend_from_slice(&chunk[..read]);

        while let Some((args, consumed)) = parse_command(&buffer) {
            buffer.drain(..consumed);
            if args.is_empty() {
                continue;
            }

            let name = String::from_utf8_lossy(&args[0]).to_ascii_uppercase();
            let decoded: Vec<String> = args[1..]
                .iter()
                .map(|arg| String::from_utf8_lossy(arg).into_owned())
                .collect();
            state
                .received
                .lock()
                .expect("received lock")
                .push(ReceivedCommand {
                    name: name.clone(),
                    args: decoded,
                });

            let reply = dispatch(&name, &args[1..], &state);
            if stream.write_all(&reply).await.is_err() {
                return;
            }
        }

        if stream.flush().await.is_err() {
            return;
        }
    }
}

fn dispatch(name: &str, args: &[Vec<u8>], state: &ServerState) -> Vec<u8> {
    if let Some(failure) = state.failure.lock().expect("failure lock").clone()
        && failure.command == name
    {
        return error_reply(&failure.message);
    }

    match name {
        "PING" => match args.first() {
            Some(payload) => bulk_string(payload),
            None => {
                let reply = state.ping_reply.lock().expect("ping lock").clone();
                simple_string(&reply)
            }
        },
        "CLIENT" => simple_string("OK"),
        "SELECT" => simple_string("OK"),
        "SETEX" => {
            let (key, ttl, value) = (
                match args.first() {
                    Some(key) => key.clone(),
                    None => return error_reply("wrong number of arguments"),
                },
                match args.get(1) {
                    Some(ttl) => ttl.clone(),
                    None => return error_reply("wrong number of arguments"),
                },
                match args.get(2) {
                    Some(value) => value.clone(),
                    None => return error_reply("wrong number of arguments"),
                },
            );
            let ttl = String::from_utf8_lossy(&ttl).parse::<i64>().unwrap_or(-1);
            let key = String::from_utf8_lossy(&key).into_owned();
            state
                .setex_ttls
                .lock()
                .expect("setex lock")
                .push((key.clone(), ttl));
            if ttl == 0 {
                state.values.lock().expect("values lock").remove(&key);
            } else {
                state
                    .values
                    .lock()
                    .expect("values lock")
                    .insert(key, String::from_utf8_lossy(&value).into_owned());
            }
            simple_string("OK")
        }
        "SET" => {
            let key = match args.first() {
                Some(key) => String::from_utf8_lossy(key).into_owned(),
                None => return error_reply("wrong number of arguments"),
            };
            let value = match args.get(1) {
                Some(value) => String::from_utf8_lossy(value).into_owned(),
                None => return error_reply("wrong number of arguments"),
            };
            state.values.lock().expect("values lock").insert(key, value);
            simple_string("OK")
        }
        "GET" => {
            let key = match args.first() {
                Some(key) => String::from_utf8_lossy(key).into_owned(),
                None => return error_reply("wrong number of arguments"),
            };
            match state.values.lock().expect("values lock").get(&key) {
                Some(value) => bulk_string(value.as_bytes()),
                None => null_bulk_string(),
            }
        }
        "INCR" | "INCRBY" => {
            let key = match args.first() {
                Some(key) => String::from_utf8_lossy(key).into_owned(),
                None => return error_reply("wrong number of arguments"),
            };
            let increment = match args.get(1) {
                Some(amount) => String::from_utf8_lossy(amount).parse::<i64>().unwrap_or(1),
                None => 1,
            };
            let mut values = state.values.lock().expect("values lock");
            let current = values
                .get(&key)
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(0);
            let next = current + increment;
            values.insert(key, next.to_string());
            integer_reply(next)
        }
        "EXPIRE" => {
            let key = match args.first() {
                Some(key) => String::from_utf8_lossy(key).into_owned(),
                None => return error_reply("wrong number of arguments"),
            };
            let exists = state.values.lock().expect("values lock").contains_key(&key);
            integer_reply(i64::from(exists))
        }
        "DEL" => {
            let mut values = state.values.lock().expect("values lock");
            let removed = args
                .iter()
                .filter(|key| {
                    values
                        .remove(String::from_utf8_lossy(key).as_ref())
                        .is_some()
                })
                .count() as i64;
            integer_reply(removed)
        }
        "EXISTS" => {
            let values = state.values.lock().expect("values lock");
            let found = args
                .iter()
                .filter(|key| values.contains_key(String::from_utf8_lossy(key).as_ref()))
                .count() as i64;
            integer_reply(found)
        }
        other => error_reply(&format!("unknown command '{other}'")),
    }
}

fn simple_string(text: &str) -> Vec<u8> {
    format!("+{text}\r\n").into_bytes()
}

fn error_reply(message: &str) -> Vec<u8> {
    format!("-ERR {message}\r\n").into_bytes()
}

fn integer_reply(value: i64) -> Vec<u8> {
    format!(":{value}\r\n").into_bytes()
}

fn bulk_string(bytes: &[u8]) -> Vec<u8> {
    let mut out = format!("${}\r\n", bytes.len()).into_bytes();
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\r\n");
    out
}

fn null_bulk_string() -> Vec<u8> {
    b"$-1\r\n".to_vec()
}

/// Parse one RESP multibulk command from the front of `buffer`.
///
/// Returns the arguments and the number of bytes consumed, or `None` when the
/// buffer does not yet hold a complete command.
fn parse_command(buffer: &[u8]) -> Option<(Vec<Vec<u8>>, usize)> {
    if buffer.first() != Some(&b'*') {
        return None;
    }
    let (arity, mut position) = read_length_line(buffer)?;
    if arity < 0 {
        return None;
    }

    let mut args = Vec::with_capacity(arity as usize);
    for _ in 0..arity {
        if buffer.get(position) != Some(&b'$') {
            return None;
        }
        let (length, header) = read_length_line(&buffer[position..])?;
        position += header;
        if length < 0 {
            args.push(Vec::new());
            continue;
        }
        let length = length as usize;
        let end = position + length;
        if buffer.len() < end + 2 {
            return None;
        }
        args.push(buffer[position..end].to_vec());
        position = end + 2;
    }

    Some((args, position))
}

/// Read `\r\n`-terminated integer at the front of `buffer` (including the `*`/`$` byte).
fn read_length_line(buffer: &[u8]) -> Option<(i64, usize)> {
    let terminator = buffer.windows(2).position(|window| window == b"\r\n")?;
    let text = std::str::from_utf8(&buffer[1..terminator]).ok()?;
    text.parse::<i64>()
        .ok()
        .map(|value| (value, terminator + 2))
}
