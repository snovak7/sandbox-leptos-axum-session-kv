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

## Architecture

This project is in early setup. The intended stack:

- **Leptos** — full-stack Rust UI framework (SSR mode with hydration)
- **Axum** — server runtime for Leptos SSR, also hosting API routes
- **Cloudflare KV** — key-value store used as the session backend (accessed via `worker` or `reqwest`-based HTTP bindings)

### Session flow (planned)

1. On login/session creation, the Axum handler writes a session record to Cloudflare KV with a TTL.
2. A session cookie carries the session ID to the browser.
3. Subsequent requests read the session from KV to authenticate/authorize.
4. Leptos server functions (`#[server]`) can access the session via Axum extractors injected into the request context.
