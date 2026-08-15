use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use std::sync::Arc;

use crate::session::{SessionData, SessionId, SessionStore};

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub session_store: Arc<dyn SessionStore + Send + Sync>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

pub type SessionExtension = Option<(SessionId, SessionData)>;
