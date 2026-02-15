use leptos::prelude::*;
use north_domain::TaskWithMeta;

use super::view::TaskDetailModalView;
use crate::server_fns::tasks::*;

type AncestorsResource = Resource<Result<Vec<(i64, String, i64)>, ServerFnError>>;
use crate::stores::task_store::TaskStore;

#[derive(Clone, Copy)]
pub struct TaskDetailContext {
    pub open_task_id: RwSignal<Option<i64>>,
}

#[component]
pub fn TaskDetailModal(task_ids: Signal<Vec<i64>>, task_store: TaskStore) -> impl IntoView {
    let ctx = expect_context::<TaskDetailContext>();

    let task_stack: RwSignal<Vec<i64>> = RwSignal::new(vec![]);

    let current_task_id = Memo::new(move |_| {
        let stack = task_stack.get();
        if let Some(top) = stack.last() {
            Some(*top)
        } else {
            ctx.open_task_id.get()
        }
    });

    let is_open = Memo::new(move |_| current_task_id.get().is_some());

    let task_detail: Resource<Result<Option<TaskWithMeta>, ServerFnError>> = Resource::new(
        move || current_task_id.get(),
        |id| async move {
            match id {
                Some(id) => get_task_detail(id).await.map(Some),
                None => Ok(None),
            }
        },
    );

    let ancestors: AncestorsResource = Resource::new(
        move || current_task_id.get(),
        |id| async move {
            match id {
                Some(id) => get_task_ancestors(id).await,
                None => Ok(vec![]),
            }
        },
    );

    let on_close = Callback::new(move |()| {
        ctx.open_task_id.set(None);
        task_stack.set(vec![]);
    });

    let on_prev = Callback::new(move |()| {
        if !task_stack.get_untracked().is_empty() {
            return;
        }
        let ids = task_ids.get_untracked();
        if let Some(current) = ctx.open_task_id.get_untracked() {
            if let Some(idx) = ids.iter().position(|&id| id == current) {
                if idx > 0 {
                    ctx.open_task_id.set(Some(ids[idx - 1]));
                }
            }
        }
    });

    let on_next = Callback::new(move |()| {
        if !task_stack.get_untracked().is_empty() {
            return;
        }
        let ids = task_ids.get_untracked();
        if let Some(current) = ctx.open_task_id.get_untracked() {
            if let Some(idx) = ids.iter().position(|&id| id == current) {
                if idx + 1 < ids.len() {
                    ctx.open_task_id.set(Some(ids[idx + 1]));
                }
            }
        }
    });

    let on_navigate_to_subtask = Callback::new(move |subtask_id: i64| {
        if let Some(current) = current_task_id.get_untracked() {
            task_stack.update(|s| s.push(current));
        }
        task_stack.update(|s| s.push(subtask_id));
    });

    let on_navigate_to_ancestor = Callback::new(move |ancestor_id: i64| {
        task_stack.update(|s| {
            // Pop stack back to ancestor
            if let Some(pos) = s.iter().position(|&id| id == ancestor_id) {
                s.truncate(pos + 1);
            } else {
                // Ancestor is the root task from open_task_id
                s.clear();
                // If the ancestor is the currently opened root task, we're done
                // Otherwise set it as the root
            }
        });
    });

    let has_stack = Memo::new(move |_| !task_stack.get().is_empty());

    // Modal-only actions
    let due_date_action = Action::new(|input: &(i64, String)| {
        let (id, due_date) = input.clone();
        set_task_due_date(id, due_date)
    });

    let clear_due_date_action = Action::new(|id: &i64| {
        let id = *id;
        clear_task_due_date(id)
    });

    let seq_limit_action = Action::new(|input: &(i64, i16)| {
        let (id, limit) = *input;
        set_sequential_limit(id, limit)
    });

    // Refetch task detail after modal-only actions
    Effect::new(move || {
        if let Some(Ok(_)) = due_date_action.value().get() {
            task_detail.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = clear_due_date_action.value().get() {
            task_detail.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = seq_limit_action.value().get() {
            task_detail.refetch();
        }
    });

    let on_set_due_date = Callback::new(move |(id, date): (i64, String)| {
        due_date_action.dispatch((id, date));
    });

    let on_clear_due_date = Callback::new(move |id: i64| {
        clear_due_date_action.dispatch(id);
    });

    let on_set_seq_limit = Callback::new(move |(id, limit): (i64, i16)| {
        seq_limit_action.dispatch((id, limit));
    });

    // Wrap task_store callbacks to also refetch task detail
    let on_toggle_complete = Callback::new(move |(id, was_completed): (i64, bool)| {
        task_store.on_toggle_complete.run((id, was_completed));
        // Refetch detail after a small delay (the store's effect will refetch
        // the list; we also need to refetch the detail)
        task_detail.refetch();
    });

    let on_delete = Callback::new(move |id: i64| {
        task_store.on_delete.run(id);
        on_close.run(());
    });

    let on_update = Callback::new(move |(id, title, body): (i64, String, Option<String>)| {
        task_store.on_update.run((id, title, body));
    });

    let on_set_start_at = Callback::new(move |(id, start_at): (i64, String)| {
        task_store.on_set_start_at.run((id, start_at));
        task_detail.refetch();
    });

    let on_clear_start_at = Callback::new(move |id: i64| {
        task_store.on_clear_start_at.run(id);
        task_detail.refetch();
    });

    let on_set_project = Callback::new(move |(task_id, project_id): (i64, i64)| {
        task_store.on_set_project.run((task_id, project_id));
        task_detail.refetch();
    });

    let on_clear_project = Callback::new(move |id: i64| {
        task_store.on_clear_project.run(id);
        task_detail.refetch();
    });

    let on_set_tags = Callback::new(move |(task_id, tags): (i64, Vec<String>)| {
        task_store.on_set_tags.run((task_id, tags));
        task_detail.refetch();
    });

    let on_refetch_detail = Callback::new(move |()| {
        task_detail.refetch();
    });

    view! {
        <Show when=move || is_open.get()>
            <TaskDetailModalView
                task_detail=task_detail
                ancestors=ancestors
                has_stack=has_stack
                on_close=on_close
                on_prev=on_prev
                on_next=on_next
                on_navigate_to_subtask=on_navigate_to_subtask
                on_navigate_to_ancestor=on_navigate_to_ancestor
                on_toggle_complete=on_toggle_complete
                on_delete=on_delete
                on_update=on_update
                on_set_start_at=on_set_start_at
                on_clear_start_at=on_clear_start_at
                on_set_project=on_set_project
                on_clear_project=on_clear_project
                on_set_tags=on_set_tags
                on_set_due_date=on_set_due_date
                on_clear_due_date=on_clear_due_date
                on_set_seq_limit=on_set_seq_limit
                on_refetch_detail=on_refetch_detail
            />
        </Show>
    }
}
