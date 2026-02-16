use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::Tag;
use north_repositories::TagRepository;

#[derive(Clone, Copy)]
pub struct TagStore {
    tags: RwSignal<Vec<Tag>>,
    loaded: RwSignal<bool>,
}

impl Default for TagStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TagStore {
    pub fn new() -> Self {
        Self {
            tags: RwSignal::new(vec![]),
            loaded: RwSignal::new(false),
        }
    }

    pub fn refetch(&self) {
        let store = *self;
        spawn_local(async move {
            if let Ok(list) = TagRepository::list().await {
                store.load(list);
            }
        });
    }

    pub fn load(&self, tags: Vec<Tag>) {
        self.tags.set(tags);
        self.loaded.set(true);
    }

    pub fn get(&self) -> Vec<Tag> {
        self.tags.get()
    }
}
