use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[server(Login)]
pub async fn login(username: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "standalone")]
    {
        use crate::{
            session::{new_session_id, SessionData},
            state::AppState,
        };
        use axum::http::HeaderValue;
        use leptos_axum::ResponseOptions;
        use std::time::Duration;

        let _ = password; // demo: accept any password

        let state = expect_context::<AppState>();
        let response = expect_context::<ResponseOptions>();

        let id = new_session_id();
        let data = SessionData { username };

        state
            .session_store
            .create(&id, &data, Duration::from_secs(86400))
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let cookie = format!(
            "session_id={}; HttpOnly; Secure; SameSite=Strict; Max-Age=86400; Path=/",
            id.0
        );
        response.insert_header(
            axum::http::header::SET_COOKIE,
            HeaderValue::from_str(&cookie).unwrap(),
        );
    }

    #[cfg(not(feature = "standalone"))]
    let _ = (username, password);

    Ok(())
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<Login>::new();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(())) = login_action.value().get() {
            navigate("/", Default::default());
        }
    });

    view! {
        <h1>"Sign In"</h1>
        <ActionForm action=login_action>
            <label>
                "Username"
                <input type="text" name="username" required placeholder="your-username"/>
            </label>
            <label>
                "Password"
                <input type="password" name="password" required placeholder="••••••••"/>
            </label>
            {move || {
                login_action.value().get().and_then(|r| r.err()).map(|e| {
                    view! { <p class="error">{e.to_string()}</p> }
                })
            }}
            <button type="submit" disabled=move || login_action.pending().get()>
                {move || if login_action.pending().get() { "Signing in…" } else { "Sign in" }}
            </button>
        </ActionForm>
    }
}
