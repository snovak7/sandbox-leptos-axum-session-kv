use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{
    session::SessionId,
    state::{AppState, SessionExtension},
};

pub async fn session_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let session_id = extract_session_cookie(request.headers());

    let session_ext: SessionExtension = match session_id {
        None => None,
        Some(id) => {
            let sid = SessionId(id);
            match state.session_store.load(&sid).await {
                Ok(Some(data)) => Some((sid, data)),
                Ok(None) => None,
                Err(e) => {
                    tracing::warn!("session load error: {e}");
                    None
                }
            }
        }
    };

    request.extensions_mut().insert(session_ext);
    next.run(request).await
}

fn extract_session_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;
    for part in cookie_str.split(';') {
        let trimmed = part.trim();
        if let Some(val) = trimmed.strip_prefix("session_id=") {
            if val.len() > 128 {
                return None;
            }
            return Some(val.to_string());
        }
    }
    None
}
