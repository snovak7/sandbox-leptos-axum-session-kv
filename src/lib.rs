pub mod app;
pub mod components;

#[cfg(feature = "ssr")]
pub mod session;

// Axum-specific plumbing — not needed for Cloudflare Workers path
#[cfg(feature = "standalone")]
pub mod middleware;
#[cfg(feature = "standalone")]
pub mod state;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    leptos::mount::hydrate_body(App);
}
