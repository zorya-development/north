use leptos::prelude::*;

use super::view::SubtaskFormView;
use crate::server_fns::tasks::create_subtask;

#[component]
pub fn SubtaskForm(
    parent_id: i64,
    project_id: Option<i64>,
    depth: usize,
    on_created: Callback<()>,
) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);
    let (title, set_title) = signal(String::new());

    let create_action = Action::new(move |input: &(i64, String, Option<i64>)| {
        let (pid, t, proj) = input.clone();
        create_subtask(pid, t, proj)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = create_action.value().get() {
            set_title.set(String::new());
            on_created.run(());
        }
    });

    let on_submit = Callback::new(move |()| {
        let t = title.get_untracked();
        if t.trim().is_empty() {
            return;
        }
        create_action.dispatch((parent_id, t, project_id));
    });

    let on_cancel = Callback::new(move |()| {
        set_expanded.set(false);
        set_title.set(String::new());
    });

    view! {
        <SubtaskFormView
            expanded=expanded
            set_expanded=set_expanded
            title=title
            set_title=set_title
            depth=depth
            on_submit=on_submit
            on_cancel=on_cancel
        />
    }
}
