use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::components::{HomePage, LoginPage};

/// Full HTML shell — passed as a plain function to `file_and_error_handler`
/// and to `leptos_routes_with_context` so it is not a `#[component]`.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
                <Stylesheet id="leptos" href="/pkg/leptos-axum-session-kv.css"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Session Demo"/>
        <Router>
            <main>
                <Routes fallback=|| view! { <p>"Page not found."</p> }>
                    <Route path=StaticSegment("/") view=HomePage/>
                    <Route path=StaticSegment("/login") view=LoginPage/>
                </Routes>
            </main>
        </Router>
    }
}
