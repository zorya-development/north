use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::SavedFilter;
use north_repositories::FilterRepository;

#[derive(Clone, Copy)]
pub struct SavedFilterStore {
    filters: RwSignal<Vec<SavedFilter>>,
    loaded: RwSignal<bool>,
}

impl Default for SavedFilterStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SavedFilterStore {
    pub fn new() -> Self {
        Self {
            filters: RwSignal::new(vec![]),
            loaded: RwSignal::new(false),
        }
    }

    pub fn refetch(&self) {
        let store = *self;
        spawn_local(async move {
            if let Ok(list) = FilterRepository::list().await {
                store.load(list);
            }
        });
    }

    pub fn load(&self, filters: Vec<SavedFilter>) {
        self.filters.set(filters);
        self.loaded.set(true);
    }

    pub fn get(&self) -> Vec<SavedFilter> {
        self.filters.get()
    }

    pub fn get_by_id(&self, id: i64) -> Option<SavedFilter> {
        self.filters
            .get_untracked()
            .into_iter()
            .find(|f| f.id == id)
    }

    pub fn add(&self, filter: SavedFilter) {
        self.filters.update(|list| list.push(filter));
    }

    pub fn remove(&self, id: i64) {
        self.filters.update(|list| list.retain(|f| f.id != id));
    }

    pub fn update_in_place(&self, id: i64, f: impl FnOnce(&mut SavedFilter)) {
        self.filters.update(|list| {
            if let Some(item) = list.iter_mut().find(|item| item.id == id) {
                f(item);
            }
        });
    }

    pub fn create(&self, title: String, query: String, on_created: Option<Callback<SavedFilter>>) {
        let store = *self;
        spawn_local(async move {
            if let Ok(filter) = FilterRepository::create(title, query).await {
                store.add(filter.clone());
                if let Some(cb) = on_created {
                    cb.run(filter);
                }
            }
        });
    }

    pub fn update(&self, id: i64, title: Option<String>, query: Option<String>) {
        let store = *self;
        if let Some(t) = &title {
            let t = t.clone();
            store.update_in_place(id, move |f| f.title = t);
        }
        if let Some(q) = &query {
            let q = q.clone();
            store.update_in_place(id, move |f| f.query = q);
        }
        spawn_local(async move {
            let _ = FilterRepository::update(id, title, query).await;
        });
    }

    pub fn delete(&self, id: i64) {
        self.remove(id);
        spawn_local(async move {
            let _ = FilterRepository::delete(id).await;
        });
    }
}
