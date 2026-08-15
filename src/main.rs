#[cfg(feature = "standalone")]
#[tokio::main]
async fn main() {
    use axum::{middleware, Router};
    use deadpool_redis::{Config, Runtime};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_axum_session_kv::{
        app::{shell, App},
        middleware::session_middleware,
        session::redis::RedisSessionStore,
        state::AppState,
    };
    use std::sync::Arc;

    tracing_subscriber::fmt::init();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
    let cfg = Config::from_url(redis_url);
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("failed to create Redis pool");

    let session_store: Arc<dyn leptos_axum_session_kv::session::SessionStore + Send + Sync> =
        Arc::new(RedisSessionStore::new(pool));

    let conf = get_configuration(None).expect("failed to read Leptos config");
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;

    let routes = generate_route_list(App);

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        session_store,
    };

    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let state = app_state.clone();
                move || provide_context(state.clone())
            },
            {
                let opts = leptos_options.clone();
                move || shell(opts.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            session_middleware,
        ))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await.expect("server error");
}

#[cfg(not(feature = "standalone"))]
fn main() {}
