use leptos::prelude::*;
use leptos::task::spawn_local;
use north_domain::{CreateProject, Project, ProjectFilter, ProjectStatus, UpdateProject};
use north_repositories::ProjectRepository;

#[derive(Clone, Copy)]
pub struct ProjectStore {
    projects: RwSignal<Vec<Project>>,
    loaded: RwSignal<bool>,
}

impl Default for ProjectStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectStore {
    pub fn new() -> Self {
        Self {
            projects: RwSignal::new(vec![]),
            loaded: RwSignal::new(false),
        }
    }

    pub fn refetch(&self) {
        let store = *self;
        spawn_local(async move {
            let filter = ProjectFilter {
                status: Some(ProjectStatus::Active),
            };
            if let Ok(list) = ProjectRepository::list(filter).await {
                store.load(list);
            }
        });
    }

    pub fn load(&self, projects: Vec<Project>) {
        self.projects.set(projects);
        self.loaded.set(true);
    }

    pub fn get(&self) -> Vec<Project> {
        self.projects.get()
    }

    pub fn create(&self, title: String) {
        let store = *self;
        spawn_local(async move {
            let input = CreateProject {
                title,
                description: None,
                view_type: None,
            };
            if let Ok(project) = ProjectRepository::create(input).await {
                store.projects.update(|list| list.push(project));
            }
        });
    }

    pub fn archive(&self, id: i64) {
        let store = *self;
        store.projects.update(|list| list.retain(|p| p.id != id));
        spawn_local(async move {
            let input = UpdateProject {
                status: Some(ProjectStatus::Archived),
                ..Default::default()
            };
            let _ = ProjectRepository::update(id, input).await;
        });
    }

    pub fn update_details(&self, id: i64, title: String, color: String) {
        let store = *self;
        store.projects.update(|list| {
            if let Some(p) = list.iter_mut().find(|p| p.id == id) {
                p.title = title.clone();
                p.color = color.clone();
            }
        });
        spawn_local(async move {
            let input = UpdateProject {
                title: Some(title),
                color: Some(color),
                ..Default::default()
            };
            let _ = ProjectRepository::update(id, input).await;
        });
    }

    pub fn delete(&self, id: i64) {
        self.projects.update(|list| list.retain(|p| p.id != id));
        spawn_local(async move {
            let _ = ProjectRepository::delete(id).await;
        });
    }
}
