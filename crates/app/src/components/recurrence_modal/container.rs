use leptos::prelude::*;
use north_dto::RecurrenceType;

use super::controller::RecurrenceModalController;
use super::view::RecurrenceModalView;

#[component]
pub fn RecurrenceModal(
    recurrence_type: Option<RecurrenceType>,
    recurrence_rule: Option<String>,
    on_save: Callback<(Option<RecurrenceType>, Option<String>)>,
    on_close: Callback<()>,
) -> impl IntoView {
    let ctrl = RecurrenceModalController::new(recurrence_type, recurrence_rule);

    view! {
        <RecurrenceModalView
            ctrl=ctrl
            on_save=Callback::new(move |()| {
                on_save.run(ctrl.save_result());
            })
            on_remove=Callback::new(move |()| {
                on_save.run(RecurrenceModalController::remove_result());
            })
            on_close=on_close
        />
    }
}
