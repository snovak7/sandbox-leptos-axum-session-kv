# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Sandbox for experimenting with session management backed by Cloudflare KV, using a Leptos frontend and Axum backend.

## Commands

> These will be populated once Cargo.toml and the project structure are in place.

Standard Rust/Leptos commands to add as the project grows:

```sh
cargo build                  # build
cargo test                   # run all tests
cargo test <test_name>       # run a single test
cargo clippy -- -D warnings  # lint
cargo fmt                    # format
```

For Leptos (SSR with Axum), builds typically go through `cargo-leptos`:

```sh
cargo leptos build           # full build (client + server)
cargo leptos watch           # dev server with hot-reload
cargo leptos test            # run tests
```

### Local environment variables

Required when running with the `standalone` feature (reqwest-based KV REST API):

```sh
CF_ACCOUNT_ID=<cloudflare-account-id>
CF_KV_NAMESPACE_ID=<kv-namespace-id>
CF_API_TOKEN=<cloudflare-api-token>   # needs KV read+write permissions
```

Store these in a `.env` file that is listed in `.gitignore` — never commit them to source control.

These map to the Cloudflare KV REST API (`https://api.cloudflare.com/client/v4/accounts/{account_id}/storage/kv/namespaces/{namespace_id}`). When wrangler.jsonc is added, its binding name (e.g. `MY_KV`) maps to the KV namespace at deploy time; `CF_KV_NAMESPACE_ID` (the UUID) is still required for the `standalone`/reqwest path.

## Architecture

This project is in early setup. The intended stack:

- **Leptos** — full-stack Rust UI framework (SSR mode with hydration)
- **Axum** — server runtime for Leptos SSR, also hosting API routes
- **Cloudflare KV** — key-value store used as the session backend, abstracted behind a `SessionStore` trait with two feature-gated implementations:
  - `cloudflare` feature — uses the `worker` crate (targets Cloudflare Workers runtime)
  - `standalone` feature (default) — uses `reqwest`-based HTTP calls to the KV REST API (targets a standard server deployment)

### Session flow (planned)

1. On login/session creation, the Axum handler generates a session ID and writes a session record to Cloudflare KV with a TTL.
2. A session cookie carries the session ID to the browser.
3. Subsequent requests read the session from KV to authenticate/authorize — always through the `SessionStore` trait, regardless of the active feature/implementation.

### Session ID requirements

- Generated using a cryptographically secure RNG, exactly **512 bits** of entropy (Base58-encoded 87–88 chars, well within the 4 KB cookie limit).
- Encoded as **Base58** to avoid special characters that could cause misrepresentation in headers or URLs.
- Session cookie must be set with `HttpOnly`, `Secure`, and `SameSite=Strict` flags.
- TTL: 24 hours by default, configurable; fixed expiry (not sliding).
