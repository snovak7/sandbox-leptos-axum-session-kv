# Session Management Prototype Plan

## Context
Greenfield Rust project (no source yet). Build a Leptos + Axum prototype demonstrating session management behind a `SessionStore` trait with two feature-gated backends:
- `standalone` (default): Redis via `deadpool-redis` — runnable locally
- `cloudflare`: Cloudflare Workers KV via `worker` crate — deploys to Workers

Note: User explicitly requested Redis for standalone, overriding the CLAUDE.md reqwest/CF-REST path.

## Goal
Working prototype with login, session cookie, authenticated home page, and logout — both session backends share one trait so swapping is a compile-time feature flag.

---

## File List

| File | Action | Purpose |
|------|--------|---------|
| `Cargo.toml` | Create | Single-crate manifest; features, profiles, `[package.metadata.leptos]` |
| `build.rs` | Create | `compile_error!` if both `standalone` and `cloudflare` are enabled |
| `Leptos.toml` | Create | cargo-leptos build config |
| `wrangler.jsonc` | Create | CF Workers deployment; `SESSION_KV` namespace binding |
| `.env.example` | Create | Template env vars for both backends |
| `style/main.scss` | Create | Minimal styles |
| `src/lib.rs` | Create | WASM `hydrate()` entry + module declarations |
| `src/main.rs` | Create | Axum server entry (standalone) |
| `src/app.rs` | Create | `App` root Leptos component + `<Router>` with `/` and `/login` routes |
| `src/state.rs` | Create | `AppState { leptos_options, session_store }` + `FromRef` impl |
| `src/session/mod.rs` | Create | `SessionId`, `SessionData`, `SessionError`, `SessionStore` trait, `new_session_id()` |
| `src/session/redis.rs` | Create | `RedisSessionStore` using `deadpool-redis` (`standalone` feature) |
| `src/session/cf_kv.rs` | Create | `CloudflareKvStore` using `worker::kv::KvStore` (`cloudflare` feature) |
| `src/middleware/mod.rs` | Create | Re-exports |
| `src/middleware/session_layer.rs` | Create | Tower middleware: reads cookie → loads session → injects into extensions |
| `src/components/mod.rs` | Create | Re-exports |
| `src/components/login_page.rs` | Create | `LoginPage` + `login()` server function |
| `src/components/home_page.rs` | Create | `HomePage` + `logout()` + `get_session_info()` server functions |

---

## Feature Architecture (`Cargo.toml`)

```toml
[features]
default  = ["standalone"]
standalone = ["dep:deadpool-redis", "dep:redis", "dep:tokio", "dep:axum", "dep:leptos_axum", "ssr"]
cloudflare = ["dep:worker", "ssr"]
ssr      = ["leptos/ssr", "leptos_router/ssr", "leptos_meta/ssr"]
hydrate  = ["leptos/hydrate", "leptos_router/hydrate", "leptos_meta/hydrate"]

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos        = { version = "0.8.19", features = ["nonce"] }
leptos_router = { version = "0.8.13" }
leptos_meta   = { version = "0.8.6" }
leptos_axum   = { version = "0.8.9",  optional = true }
axum          = { version = "0.8.9",  optional = true }
axum-extra    = { version = "0.12.6", features = ["cookie"], optional = true }
tokio         = { version = "1.52",   features = ["full"], optional = true }
serde         = { version = "1.0.228", features = ["derive"] }
serde_json    = "1.0.149"
async-trait   = "0.1.89"
bs58          = "0.5.1"
rand          = { version = "0.8.6", features = ["getrandom"] }
thiserror     = "2.0.18"
time          = { version = "0.3.47", features = ["serde"] }
tower         = { version = "0.5.3",  optional = true }
tower-http    = { version = "0.6.8",  features = ["fs"], optional = true }
tracing       = "0.1.44"
tracing-subscriber = { version = "0.3.23", optional = true }

# Standalone backend
deadpool-redis = { version = "0.23.0", optional = true }
redis          = { version = "1.2.0",  features = ["tokio-comp"], optional = true }

# Cloudflare Workers backend
worker = { version = "0.8.1", optional = true }
```

---

## Key Implementation Details

### `SessionStore` Trait (`src/session/mod.rs`)

```rust
#[cfg_attr(feature = "standalone", async_trait::async_trait)]
#[cfg_attr(feature = "cloudflare", async_trait::async_trait(?Send))]
pub trait SessionStore {
    async fn create(&self, id: &SessionId, data: &SessionData, ttl: Duration) -> Result<(), SessionError>;
    async fn load(&self, id: &SessionId) -> Result<Option<SessionData>, SessionError>;
    async fn destroy(&self, id: &SessionId) -> Result<(), SessionError>;
}
```

`?Send` for `cloudflare` (wasm32 single-threaded); `Send + Sync` for `standalone` (Axum).

### Session ID — 512-bit entropy, Base58

```rust
pub fn new_session_id() -> SessionId {
    let mut bytes = [0u8; 64]; // 512 bits
    rand::thread_rng().fill_bytes(&mut bytes);
    SessionId(bs58::encode(bytes).into_string())  // 87–88 chars, no special chars
}
```

### Redis Backend (`src/session/redis.rs`)
- `RedisSessionStore { pool: deadpool_redis::Pool }`
- `SETEX session:{id} {ttl_secs} {json}` / `GET` / `DEL`

### Cloudflare KV Backend (`src/session/cf_kv.rs`)
- `CloudflareKvStore { kv: worker::kv::KvStore }`
- `kv.put(key, json)?.expiration_ttl(secs).execute().await` / `.get().text().await` / `.delete().await`

### Session Cookie
```
session_id=<88chars>; HttpOnly; Secure; SameSite=Strict; Max-Age=86400; Path=/
```
Set via `leptos_axum::ResponseOptions::insert_header`.

### State Injection into Server Functions
```rust
.leptos_routes_with_context(
    &app_state, routes,
    { let s = app_state.clone(); move || provide_context(s.clone()) },
    App,
)
```
Server functions: `expect_context::<AppState>()` + `leptos_axum::extract::<Extension<Option<(SessionId, SessionData)>>>()`.

### Session Middleware
`axum::middleware::from_fn_with_state`: reads `session_id` cookie → `store.load()` → inserts `Option<(SessionId, SessionData)>` into request extensions.

---

## Implementation Order

1. Project scaffold — `Cargo.toml`, `build.rs`, `Leptos.toml`, config files
2. Session core — `src/session/mod.rs` (types, trait, ID generation)
3. Redis backend — `src/session/redis.rs`
4. Cloudflare KV backend — `src/session/cf_kv.rs`
5. Server plumbing — `src/state.rs`, middleware
6. Leptos UI — `src/app.rs`, components
7. Entry points — `src/lib.rs`, `src/main.rs`

---

## Acceptance Criteria

- `cargo check --no-default-features --features standalone` passes
- `cargo check --no-default-features --features cloudflare --target wasm32-unknown-unknown` passes
- `cargo check --no-default-features --features standalone,cloudflare` fails (build.rs panic)
- `cargo check --no-default-features --features hydrate --target wasm32-unknown-unknown` passes
- Session cookie has HttpOnly, Secure, SameSite=Strict flags
- Login creates session in Redis; session survives page reload
- Logout deletes session from Redis and clears cookie

---

## Verification

```sh
# Compile checks
cargo check --no-default-features --features standalone
cargo check --no-default-features --features hydrate --target wasm32-unknown-unknown

# Run locally (Redis required)
docker run --rm -p 6379:6379 redis:alpine
REDIS_URL=redis://127.0.0.1:6379 cargo leptos watch
# Visit http://localhost:3000
```

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| `worker` crate requires `wasm32-unknown-unknown` toolchain | Gate all usage with `#[cfg(feature = "cloudflare")]` |
| Leptos 0.8 server function extractor API may differ from older examples | Keep server functions simple; follow `leptos_axum::extract` pattern from 0.8 docs |
| `deadpool-redis` 0.23 + `redis` 1.x API changes from older examples | Pin to exact versions in Cargo.toml; verify at check time |
| Cookie `Secure` flag on localhost | Browsers allow `Secure` cookies on localhost; acceptable for prototype |
