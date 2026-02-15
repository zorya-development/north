use leptos::prelude::*;
use leptos::task::spawn_local;
use north_domain::Task;
use north_repositories::FilterRepository;
use north_stores::{AppStore, TaskDetailModalStore};

#[derive(Clone, Copy)]
pub struct FilterController {
    app_store: AppStore,
    task_detail_modal_store: TaskDetailModalStore,
    navigate: Callback<String>,
    pub filter_id: Memo<Option<i64>>,
    pub query_text: (ReadSignal<String>, WriteSignal<String>),
    pub committed_query: (ReadSignal<String>, WriteSignal<String>),
    pub title_text: (ReadSignal<String>, WriteSignal<String>),
    pub original_title: (ReadSignal<String>, WriteSignal<String>),
    pub original_query: (ReadSignal<String>, WriteSignal<String>),
    pub parse_error: (ReadSignal<Option<String>>, WriteSignal<Option<String>>),
    pub is_editing_title: (ReadSignal<bool>, WriteSignal<bool>),
    pub show_save_modal: (ReadSignal<bool>, WriteSignal<bool>),
    pub modal_title: (ReadSignal<String>, WriteSignal<String>),
    pub is_dirty: Memo<bool>,
    pub filter_result_ids: Memo<Vec<i64>>,
    pub is_loaded: Signal<bool>,
}

impl FilterController {
    pub fn new(
        app_store: AppStore,
        filter_id: Memo<Option<i64>>,
        navigate: Callback<String>,
    ) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;

        let query_text = signal(String::new());
        let committed_query = signal(String::new());
        let title_text = signal("Untitled Filter".to_string());
        let original_title = signal(String::new());
        let original_query = signal(String::new());
        let parse_error = signal(Option::<String>::None);
        let is_editing_title = signal(false);
        let show_save_modal = signal(false);
        let modal_title = signal(String::new());

        let result_ids = RwSignal::new(Vec::<i64>::new());
        let result_tasks = RwSignal::new(Vec::<Task>::new());
        let loaded = RwSignal::new(true);

        let is_dirty = Memo::new(move |_| {
            title_text.0.get() != original_title.0.get()
                || query_text.0.get() != original_query.0.get()
        });

        // Load existing filter if editing
        Effect::new(move |_| {
            if let Some(id) = filter_id.get() {
                if let Some(f) = app_store.saved_filters.get_by_id(id) {
                    title_text.1.set(f.title.clone());
                    query_text.1.set(f.query.clone());
                    committed_query.1.set(f.query.clone());
                    original_title.1.set(f.title);
                    original_query.1.set(f.query);
                    is_editing_title.1.set(false);
                }
            } else {
                title_text.1.set("Untitled Filter".to_string());
                query_text.1.set(String::new());
                committed_query.1.set(String::new());
                original_title.1.set(String::new());
                original_query.1.set(String::new());
                result_ids.set(vec![]);
                result_tasks.set(vec![]);
            }
        });

        // Client-side parse validation
        Effect::new(move || {
            let q = query_text.0.get();
            if q.trim().is_empty() {
                parse_error.1.set(None);
                return;
            }
            match north_domain::parse_filter(&q) {
                Ok(_) => parse_error.1.set(None),
                Err(errs) => {
                    let msg = errs
                        .into_iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; ");
                    parse_error.1.set(Some(msg));
                }
            }
        });

        // Execute filter when committed_query changes
        Effect::new(move |_| {
            let q = committed_query.0.get();
            if q.trim().is_empty() {
                result_ids.set(vec![]);
                result_tasks.set(vec![]);
                loaded.set(true);
                return;
            }
            loaded.set(false);
            spawn_local(async move {
                match FilterRepository::execute(q).await {
                    Ok(tasks) => {
                        let ids = tasks.iter().map(|t| t.id).collect();
                        result_tasks.set(tasks);
                        result_ids.set(ids);
                    }
                    Err(_) => {
                        result_ids.set(vec![]);
                        result_tasks.set(vec![]);
                    }
                }
                loaded.set(true);
            });
        });

        let filter_result_ids = Memo::new(move |_| result_ids.get());
        let is_loaded = Signal::derive(move || loaded.get());

        Self {
            app_store,
            task_detail_modal_store,
            navigate,
            filter_id,
            query_text,
            committed_query,
            title_text,
            original_title,
            original_query,
            parse_error,
            is_editing_title,
            show_save_modal,
            modal_title,
            is_dirty,
            filter_result_ids,
            is_loaded,
        }
    }

    pub fn run_query(&self) {
        let q = self.query_text.0.get_untracked();
        if !q.trim().is_empty() && self.parse_error.0.get_untracked().is_none() {
            self.committed_query.1.set(q);
        }
    }

    pub fn save(&self) {
        let query = self.query_text.0.get_untracked();
        if query.trim().is_empty() || self.parse_error.0.get_untracked().is_some() {
            return;
        }
        if self.filter_id.get_untracked().is_none() {
            self.modal_title.1.set(String::new());
            self.show_save_modal.1.set(true);
        } else {
            let title = self.title_text.0.get_untracked();
            let id = self.filter_id.get_untracked().unwrap();
            self.app_store
                .saved_filters
                .update(id, Some(title.clone()), Some(query.clone()));
            self.original_title.1.set(title);
            self.original_query.1.set(query);
        }
    }

    pub fn save_new(&self) {
        let title = self.modal_title.0.get_untracked();
        if title.trim().is_empty() {
            return;
        }
        let query = self.query_text.0.get_untracked();
        self.show_save_modal.1.set(false);

        let navigate = self.navigate;
        let set_title = self.title_text.1;
        let set_orig_title = self.original_title.1;
        let set_orig_query = self.original_query.1;
        let set_editing = self.is_editing_title.1;

        self.app_store.saved_filters.create(
            title,
            query,
            Some(Callback::new(move |filter: north_domain::SavedFilter| {
                set_title.set(filter.title.clone());
                set_orig_title.set(filter.title);
                set_orig_query.set(filter.query);
                set_editing.set(false);
                navigate.run(format!("/filters/{}", filter.id));
            })),
        );
    }

    pub fn delete(&self) {
        if let Some(id) = self.filter_id.get_untracked() {
            self.app_store.saved_filters.delete(id);
            self.navigate.run("/filters/new".to_string());
        }
    }

    pub fn open_detail(&self, task_id: i64) {
        let task_ids = self.filter_result_ids.get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }
}
