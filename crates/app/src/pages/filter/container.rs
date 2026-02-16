use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use north_stores::use_app_store;

use super::controller::FilterController;
use super::view::FilterView;

#[component]
pub fn FilterPage() -> impl IntoView {
    let app_store = use_app_store();
    let params = use_params_map();
    let navigate = use_navigate();

    let filter_id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<i64>().ok())
    });

    let nav_cb = Callback::new(move |path: String| {
        navigate(&path, Default::default());
    });

    let ctrl = FilterController::new(app_store, filter_id, nav_cb);

    view! {
        <FilterView
            ctrl=ctrl
            on_run_query=Callback::new(move |()| ctrl.run_query())
            on_save=Callback::new(move |()| ctrl.save())
            on_save_new=Callback::new(move |()| ctrl.save_new())
            on_delete=Callback::new(move |()| ctrl.delete())
            on_task_click=Callback::new(move |id| ctrl.open_detail(id))
        />
    }
}
