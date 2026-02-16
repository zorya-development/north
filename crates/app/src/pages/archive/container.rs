use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::ArchiveController;
use super::view::ArchiveView;

#[component]
pub fn ArchivePage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = ArchiveController::new(app_store);

    view! {
        <ArchiveView
            archived_projects=ctrl.archived_projects
            is_loaded=ctrl.is_loaded
            on_unarchive=Callback::new(move |id| ctrl.unarchive(id))
            on_delete=Callback::new(move |id| ctrl.delete(id))
        />
    }
}
