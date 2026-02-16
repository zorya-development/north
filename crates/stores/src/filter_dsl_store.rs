use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::{DslSuggestion, Task};
use north_repositories::FilterRepository;

#[derive(Clone, Copy)]
pub struct FilterDslStore {
    query: RwSignal<String>,
    parse_error: RwSignal<Option<String>>,
    suggestions: RwSignal<Vec<DslSuggestion>>,
    result_tasks: RwSignal<Vec<Task>>,
    result_ids: RwSignal<Vec<i64>>,
    loaded: RwSignal<bool>,
}

impl Default for FilterDslStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterDslStore {
    pub fn new() -> Self {
        Self {
            query: RwSignal::new(String::new()),
            parse_error: RwSignal::new(None),
            suggestions: RwSignal::new(vec![]),
            result_tasks: RwSignal::new(vec![]),
            result_ids: RwSignal::new(vec![]),
            loaded: RwSignal::new(true),
        }
    }

    pub fn query(&self) -> ReadSignal<String> {
        self.query.read_only()
    }

    pub fn set_query(&self, text: String) {
        self.query.set(text.clone());
        self.validate_async(text);
    }

    pub fn parse_error(&self) -> ReadSignal<Option<String>> {
        self.parse_error.read_only()
    }

    pub fn suggestions(&self) -> ReadSignal<Vec<DslSuggestion>> {
        self.suggestions.read_only()
    }

    pub fn update_completions(&self, cursor: usize) {
        let query = self.query.get_untracked();
        let suggestions = self.suggestions;
        spawn_local(async move {
            match FilterRepository::get_completions(query, cursor).await {
                Ok(items) => suggestions.set(items),
                Err(_) => suggestions.set(vec![]),
            }
        });
    }

    pub fn clear_suggestions(&self) {
        self.suggestions.set(vec![]);
    }

    pub fn apply_completion(&self, suggestion: &DslSuggestion, cursor: usize) -> (String, usize) {
        let query = self.query.get_untracked();
        let before = &query[..suggestion.start];
        let after = &query[cursor..];
        let insertion = format!("{} ", suggestion.value);
        let new_cursor = before.len() + insertion.len();
        let new_value = format!("{before}{insertion}{after}");
        self.query.set(new_value.clone());
        self.suggestions.set(vec![]);
        self.validate_async(new_value.clone());
        (new_value, new_cursor)
    }

    pub fn execute(&self) {
        let q = self.query.get_untracked();
        if q.trim().is_empty() || self.parse_error.get_untracked().is_some() {
            return;
        }
        let result_tasks = self.result_tasks;
        let result_ids = self.result_ids;
        let loaded = self.loaded;
        loaded.set(false);
        spawn_local(async move {
            match FilterRepository::execute(q).await {
                Ok(tasks) => {
                    let ids = tasks.iter().map(|t| t.id).collect();
                    result_tasks.set(tasks);
                    result_ids.set(ids);
                }
                Err(_) => {
                    result_tasks.set(vec![]);
                    result_ids.set(vec![]);
                }
            }
            loaded.set(true);
        });
    }

    pub fn result_tasks(&self) -> ReadSignal<Vec<Task>> {
        self.result_tasks.read_only()
    }

    pub fn result_ids(&self) -> ReadSignal<Vec<i64>> {
        self.result_ids.read_only()
    }

    pub fn is_loaded(&self) -> Signal<bool> {
        let loaded = self.loaded;
        Signal::derive(move || loaded.get())
    }

    pub fn clear_results(&self) {
        self.result_tasks.set(vec![]);
        self.result_ids.set(vec![]);
    }

    pub fn reset(&self) {
        self.query.set(String::new());
        self.parse_error.set(None);
        self.suggestions.set(vec![]);
        self.result_tasks.set(vec![]);
        self.result_ids.set(vec![]);
        self.loaded.set(true);
    }

    pub fn load_query(&self, text: String) {
        self.query.set(text);
        self.parse_error.set(None);
    }

    fn validate_async(&self, text: String) {
        if text.trim().is_empty() {
            self.parse_error.set(None);
            return;
        }
        let parse_error = self.parse_error;
        spawn_local(async move {
            match FilterRepository::validate_query(text).await {
                Ok(()) => parse_error.set(None),
                Err(e) => parse_error.set(Some(e.to_string())),
            }
        });
    }
}
