use leptos::prelude::*;
use north_domain::CreateTask;
use north_stores::AppStore;

#[derive(Clone, Copy)]
pub struct TaskInlineFormController {
    app_store: AppStore,
    task_id: Option<i64>,
    pub title: (ReadSignal<String>, WriteSignal<String>),
    pub body: (ReadSignal<String>, WriteSignal<String>),
    pub preview: (ReadSignal<bool>, WriteSignal<bool>),
    on_done: Callback<()>,
}

impl TaskInlineFormController {
    pub fn new(app_store: AppStore, task_id: Option<i64>, on_done: Callback<()>) -> Self {
        let initial_title;
        let initial_body;
        if let Some(id) = task_id {
            let task = app_store.tasks.get_by_id(id);
            let t = task.get_untracked();
            initial_title = t.as_ref().map(|t| t.title.clone()).unwrap_or_default();
            initial_body = t
                .as_ref()
                .and_then(|t| t.body.clone())
                .unwrap_or_default();
        } else {
            initial_title = String::new();
            initial_body = String::new();
        }

        let title = signal(initial_title);
        let body = signal(initial_body);
        let preview = signal(false);

        Self {
            app_store,
            task_id,
            title,
            body,
            preview,
            on_done,
        }
    }

    pub fn save(&self) {
        let t = self.title.0.get_untracked().trim().to_string();
        if t.is_empty() {
            return;
        }
        let b = self.body.0.get_untracked().trim().to_string();
        let body_opt = if b.is_empty() { None } else { Some(b) };

        if let Some(id) = self.task_id {
            self.app_store.tasks.update_task(id, t, body_opt);
        } else {
            self.app_store.tasks.create_task(CreateTask {
                title: t,
                body: body_opt,
                ..Default::default()
            });
        }
        self.on_done.run(());
    }

    pub fn cancel(&self) {
        self.on_done.run(());
    }

    pub fn is_edit_mode(&self) -> bool {
        self.task_id.is_some()
    }
}
