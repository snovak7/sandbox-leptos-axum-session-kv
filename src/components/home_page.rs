use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[server(GetSessionInfo)]
pub async fn get_session_info() -> Result<Option<String>, ServerFnError> {
    #[cfg(feature = "standalone")]
    {
        use axum::Extension;
        use crate::state::SessionExtension;
        use leptos_axum::extract;

        let Extension(session): Extension<SessionExtension> = extract()
            .await
            .map_err(|_| ServerFnError::new("failed to extract session"))?;

        return Ok(session.map(|(_, data)| data.username));
    }

    #[allow(unreachable_code)]
    Ok(None)
}

#[server(Logout)]
pub async fn logout() -> Result<(), ServerFnError> {
    #[cfg(feature = "standalone")]
    {
        use axum::{http::HeaderValue, Extension};
        use crate::state::{AppState, SessionExtension};
        use leptos_axum::{extract, ResponseOptions};

        let Extension(session): Extension<SessionExtension> = extract()
            .await
            .map_err(|_| ServerFnError::new("failed to extract session"))?;

        let state = expect_context::<AppState>();
        let response = expect_context::<ResponseOptions>();

        if let Some((id, _)) = session {
            state.session_store.destroy(&id).await.map_err(|e| {
                tracing::error!("session destroy failed, server-side record may persist: {e}");
                ServerFnError::new(format!("logout incomplete: {e}"))
            })?;
        }

        response.insert_header(
            axum::http::header::SET_COOKIE,
            HeaderValue::from_static(
                "session_id=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/",
            ),
        );
    }

    Ok(())
}

#[component]
pub fn HomePage() -> impl IntoView {
    let session_info = Resource::new(|| (), |_| get_session_info());
    let logout_action = ServerAction::<Logout>::new();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(())) = logout_action.value().get() {
            session_info.refetch();
            navigate("/login", Default::default());
        }
    });

    view! {
        <h1>"Home"</h1>
        <Suspense fallback=|| view! { <p>"Loading…"</p> }>
            {move || {
                session_info.get().map(|result| {
                    match result {
                        Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                        Ok(None) => view! {
                            <p>"Not signed in. " <a href="/login">"Sign in"</a></p>
                        }.into_any(),
                        Ok(Some(username)) => view! {
                            <div class="session-info">
                                <p>"Signed in as "<strong>{username}</strong></p>
                            </div>
                            <ActionForm action=logout_action>
                                <button type="submit">"Sign out"</button>
                            </ActionForm>
                        }.into_any(),
                    }
                })
            }}
        </Suspense>
    }
}
