use leptos::prelude::*;
use north_recurrence::RecurrenceType;

use super::controller::build_reactive_rule;
use super::view::RecurrenceModalView;

#[component]
pub fn RecurrenceModal(
    recurrence_type: Option<RecurrenceType>,
    recurrence_rule: Option<String>,
    on_save: Callback<(Option<RecurrenceType>, Option<String>)>,
    on_close: Callback<()>,
) -> impl IntoView {
    let ctrl = build_reactive_rule(recurrence_type, recurrence_rule);

    view! {
        <RecurrenceModalView
            ctrl=ctrl
            on_save=Callback::new(move |()| {
                on_save.run(ctrl.to_result());
            })
            on_remove=Callback::new(move |()| {
                on_save.run((None, None));
            })
            on_close=on_close
        />
    }
}
