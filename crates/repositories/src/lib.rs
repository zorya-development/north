pub mod filter_repo;
pub mod models;
pub mod project_repo;
pub mod settings_repo;
pub mod tag_repo;
pub mod task_repo;

pub use filter_repo::FilterRepository;
pub use models::{Recurrence, TaskModel};
pub use project_repo::ProjectRepository;
pub use settings_repo::SettingsRepository;
pub use tag_repo::TagRepository;
pub use task_repo::TaskRepository;

use leptos::prelude::*;

/// Provided by AppLayout, wired to StatusBarStore::notify(Danger, ...).
/// Repositories use this to surface errors without depending on the stores crate.
#[derive(Clone)]
pub struct ErrorNotifier(pub Callback<String>);

/// Wraps a Result, notifying the user on Err via ErrorNotifier context.
/// Returns the Result unchanged so callers can still handle it.
pub fn notify_on_error<T>(result: Result<T, ServerFnError>) -> Result<T, ServerFnError> {
    if let Err(ref e) = result {
        if let Some(notifier) = use_context::<ErrorNotifier>() {
            notifier.0.run(format_user_error(e));
        }
    }
    result
}

fn format_user_error(e: &ServerFnError) -> String {
    let msg = e.to_string();
    let msg = msg.strip_prefix("ServerFnError: ").unwrap_or(&msg);
    if msg.contains("error sending request")
        || msg.contains("NetworkError")
        || msg.contains("Failed to fetch")
    {
        return "Could not reach the server".to_string();
    }
    msg.to_string()
}
