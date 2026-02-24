use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::{Project, ProjectFilter, ProjectStatus, UpdateProject};
use north_repositories::ProjectRepository;
use north_stores::AppStore;

#[derive(Clone, Copy)]
pub struct ArchiveController {
    app_store: AppStore,
    pub archived_projects: Memo<Vec<Project>>,
    pub is_loaded: Signal<bool>,
    projects: RwSignal<Vec<Project>>,
}

impl ArchiveController {
    pub fn new(app_store: AppStore) -> Self {
        let projects = RwSignal::new(Vec::<Project>::new());
        let loaded = RwSignal::new(false);

        Effect::new(move |_| {
            spawn_local(async move {
                let filter = ProjectFilter {
                    status: Some(ProjectStatus::Archived),
                };
                if let Ok(list) = ProjectRepository::list(filter).await {
                    projects.set(list);
                }
                loaded.set(true);
            });
        });

        let archived_projects = Memo::new(move |_| projects.get());
        let is_loaded = Signal::derive(move || loaded.get());

        Self {
            app_store,
            archived_projects,
            is_loaded,
            projects,
        }
    }

    pub fn unarchive(&self, id: i64) {
        let app_store = self.app_store;
        let projects = self.projects;
        projects.update(|list| list.retain(|p| p.id != id));
        spawn_local(async move {
            let input = UpdateProject {
                status: Some(ProjectStatus::Active),
                ..Default::default()
            };
            if ProjectRepository::update(id, input).await.is_ok() {
                app_store.projects.refetch();
            }
        });
    }

    pub fn delete(&self, id: i64) {
        let projects = self.projects;
        projects.update(|list| list.retain(|p| p.id != id));
        spawn_local(async move {
            let _ = ProjectRepository::delete(id).await;
        });
    }
}
