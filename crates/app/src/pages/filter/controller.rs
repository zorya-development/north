use leptos::prelude::*;
use north_stores::{AppStore, TaskDetailModalStore};

#[derive(Clone, Copy)]
pub struct FilterController {
    app_store: AppStore,
    task_detail_modal_store: TaskDetailModalStore,
    navigate: Callback<String>,
    pub filter_id: Memo<Option<i64>>,
    pub title_text: (ReadSignal<String>, WriteSignal<String>),
    pub original_title: (ReadSignal<String>, WriteSignal<String>),
    pub original_query: (ReadSignal<String>, WriteSignal<String>),
    pub is_editing_title: (ReadSignal<bool>, WriteSignal<bool>),
    pub show_save_modal: (ReadSignal<bool>, WriteSignal<bool>),
    pub modal_title: (ReadSignal<String>, WriteSignal<String>),
    pub is_dirty: Memo<bool>,
}

impl FilterController {
    pub fn new(
        app_store: AppStore,
        filter_id: Memo<Option<i64>>,
        initial_query: Memo<Option<String>>,
        navigate: Callback<String>,
    ) -> Self {
        let task_detail_modal_store = app_store.task_detail_modal;
        let filter_dsl = app_store.filter_dsl;

        let title_text = signal("Untitled Filter".to_string());
        let original_title = signal(String::new());
        let original_query = signal(String::new());
        let is_editing_title = signal(false);
        let show_save_modal = signal(false);
        let modal_title = signal(String::new());

        let is_dirty = Memo::new(move |_| {
            title_text.0.get() != original_title.0.get()
                || filter_dsl.query().get() != original_query.0.get()
        });

        // Load existing filter if editing
        Effect::new(move |_| {
            if let Some(id) = filter_id.get() {
                if let Some(f) = app_store.saved_filters.get_by_id(id) {
                    title_text.1.set(f.title.clone());
                    filter_dsl.load_query(f.query.clone());
                    original_title.1.set(f.title);
                    original_query.1.set(f.query);
                    is_editing_title.1.set(false);
                }
            } else if let Some(q) = initial_query.get() {
                title_text.1.set("Untitled Filter".to_string());
                original_title.1.set(String::new());
                original_query.1.set(String::new());
                filter_dsl.load_query(q);
                filter_dsl.execute();
            } else {
                filter_dsl.reset();
                title_text.1.set("Untitled Filter".to_string());
                original_title.1.set(String::new());
                original_query.1.set(String::new());
            }
        });

        Self {
            app_store,
            task_detail_modal_store,
            navigate,
            filter_id,
            title_text,
            original_title,
            original_query,
            is_editing_title,
            show_save_modal,
            modal_title,
            is_dirty,
        }
    }

    pub fn run_query(&self) {
        self.app_store.filter_dsl.execute();
    }

    pub fn save(&self) {
        let filter_dsl = self.app_store.filter_dsl;
        let query = filter_dsl.query().get_untracked();
        if query.trim().is_empty() || filter_dsl.parse_error().get_untracked().is_some() {
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
        let query = self.app_store.filter_dsl.query().get_untracked();
        self.show_save_modal.1.set(false);

        let navigate = self.navigate;
        let set_title = self.title_text.1;
        let set_orig_title = self.original_title.1;
        let set_orig_query = self.original_query.1;
        let set_editing = self.is_editing_title.1;

        self.app_store.saved_filters.create(
            title,
            query,
            Some(Callback::new(move |filter: north_dto::SavedFilter| {
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
        let task_ids = self.app_store.filter_dsl.result_ids().get_untracked();
        self.task_detail_modal_store.open(task_id, task_ids);
    }
}
