use leptos::prelude::*;
use north_stores::use_app_store;

use super::controller::AllTasksController;
use super::view::AllTasksView;

#[component]
pub fn AllTasksPage() -> impl IntoView {
    let app_store = use_app_store();
    let ctrl = AllTasksController::new(app_store);

    view! {
        <AllTasksView
            view=ctrl.view
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
            on_reorder=Callback::new(move |(id, key, parent)| {
                ctrl.reorder_task(id, key, parent)
            })
            toolbar=ctrl.toolbar_config()
            show_keybindings_help=RwSignal::new(false)
        />
    }
}
