use leptos::prelude::*;
use leptos::task::spawn_local;
use north_dto::CreateTask;
use north_stores::{AppStore, TaskStoreFilter};

use super::tree::*;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct TraversableTaskListController {
    pub flat_nodes: Memo<Vec<FlatNode>>,
    pub cursor_task_id: RwSignal<Option<i64>>,
    pub cursor_index: Memo<Option<usize>>,
    pub inline_mode: RwSignal<InlineMode>,
    pub create_input_value: RwSignal<String>,
    app_store: AppStore,
    on_task_click: Option<Callback<i64>>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
}

impl TraversableTaskListController {
    pub fn new(
        app_store: AppStore,
        root_task_ids: Memo<Vec<i64>>,
        show_completed: RwSignal<bool>,
        on_task_click: Option<Callback<i64>>,
        on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
    ) -> Self {
        let all_tasks = app_store.tasks.filtered(TaskStoreFilter::default());

        let flat_nodes = Memo::new(move |_| {
            let roots = root_task_ids.get();
            let tasks = all_tasks.get();
            flatten_tree(&roots, &tasks, show_completed.get())
        });

        let cursor_task_id = RwSignal::new(None::<i64>);

        let cursor_index = Memo::new(move |_| {
            let id = cursor_task_id.get()?;
            let nodes = flat_nodes.get();
            nodes.iter().position(|n| n.task_id == id)
        });

        let inline_mode = RwSignal::new(InlineMode::None);
        let create_input_value = RwSignal::new(String::new());

        Self {
            flat_nodes,
            cursor_task_id,
            cursor_index,
            inline_mode,
            create_input_value,
            app_store,
            on_task_click,
            on_reorder,
        }
    }

    // ── Cursor navigation ──────────────────────────────────────

    pub fn move_up(&self) {
        let nodes = self.flat_nodes.get_untracked();
        if let Some(id) = self.cursor_task_id.get_untracked() {
            if let Some(prev) = prev_sibling(&nodes, id) {
                self.cursor_task_id.set(Some(prev));
            }
        } else if let Some(first) = nodes.first() {
            self.cursor_task_id.set(Some(first.task_id));
        }
    }

    pub fn move_down(&self) {
        let nodes = self.flat_nodes.get_untracked();
        if let Some(id) = self.cursor_task_id.get_untracked() {
            if let Some(next) = next_sibling(&nodes, id) {
                self.cursor_task_id.set(Some(next));
            }
        } else if let Some(first) = nodes.first() {
            self.cursor_task_id.set(Some(first.task_id));
        }
    }

    pub fn move_right(&self) {
        let nodes = self.flat_nodes.get_untracked();
        if let Some(id) = self.cursor_task_id.get_untracked() {
            if let Some(child) = first_child(&nodes, id) {
                self.cursor_task_id.set(Some(child));
            }
        }
    }

    pub fn move_left(&self) {
        let nodes = self.flat_nodes.get_untracked();
        if let Some(id) = self.cursor_task_id.get_untracked() {
            if let Some(parent) = parent_of(&nodes, id) {
                self.cursor_task_id.set(Some(parent));
            }
        }
    }

    // ── Inline edit ────────────────────────────────────────────

    pub fn start_edit(&self) {
        if let Some(id) = self.cursor_task_id.get_untracked() {
            self.inline_mode.set(InlineMode::Edit { task_id: id });
        }
    }

    pub fn save_edit(&self, new_title: String) {
        if let InlineMode::Edit { task_id } = self.inline_mode.get_untracked() {
            if !new_title.is_empty() {
                let body = self
                    .app_store
                    .tasks
                    .get_by_id(task_id)
                    .get_untracked()
                    .and_then(|t| t.body);
                self.app_store.tasks.update_task(task_id, new_title, body);
            }
            self.inline_mode.set(InlineMode::None);
        }
    }

    pub fn cancel_edit(&self) {
        if matches!(self.inline_mode.get_untracked(), InlineMode::Edit { .. }) {
            self.inline_mode.set(InlineMode::None);
        }
    }

    // ── Inline create ──────────────────────────────────────────

    pub fn start_create(&self, placement: Placement) {
        if let Some(anchor_id) = self.cursor_task_id.get_untracked() {
            let nodes = self.flat_nodes.get_untracked();
            if let Some(anchor) = nodes.iter().find(|n| n.task_id == anchor_id) {
                let depth = anchor.depth;
                let parent_id = anchor.parent_id;
                self.create_input_value.set(String::new());
                self.inline_mode.set(InlineMode::Create {
                    anchor_task_id: anchor_id,
                    placement,
                    parent_id,
                    depth,
                });
            }
        }
    }

    pub fn create_task(&self) {
        let mode = self.inline_mode.get_untracked();
        let InlineMode::Create {
            anchor_task_id,
            placement,
            parent_id,
            depth,
        } = mode
        else {
            return;
        };

        let title = self.create_input_value.get_untracked().trim().to_string();
        if title.is_empty() {
            self.close_inline();
            return;
        }

        let nodes = self.flat_nodes.get_untracked();
        let all_tasks = self.app_store.tasks.filtered(TaskStoreFilter::default());
        let tasks = all_tasks.get_untracked();

        let sort_key = compute_sort_key(&nodes, &tasks, anchor_task_id, placement, parent_id);

        let project_id = parent_id.and_then(|pid| {
            tasks
                .iter()
                .find(|t| t.id == pid)
                .and_then(|t| t.project_id)
        });

        let input = CreateTask {
            title,
            parent_id,
            project_id,
            sort_key: Some(sort_key),
            ..Default::default()
        };

        self.create_input_value.set(String::new());

        let store = self.app_store.tasks;
        let inline_mode = self.inline_mode;
        spawn_local(async move {
            if let Some(task) = store.create_task_async(input).await {
                // For After placement, chain: next create goes after the
                // newly created task. For Before, anchor stays the same.
                if placement == Placement::After {
                    inline_mode.set(InlineMode::Create {
                        anchor_task_id: task.id,
                        placement: Placement::After,
                        parent_id,
                        depth,
                    });
                }
            }
        });
    }

    pub fn indent(&self) {
        self.inline_mode.update(|mode| {
            if let InlineMode::Create {
                anchor_task_id,
                placement,
                depth,
                parent_id,
            } = mode
            {
                let nodes = self.flat_nodes.get_untracked();
                let max = max_create_depth(&nodes, *anchor_task_id, *placement);
                if *depth < max {
                    *depth += 1;
                    *parent_id = find_parent_for_depth(&nodes, *anchor_task_id, *depth);
                }
            }
        });
    }

    pub fn outdent(&self) {
        self.inline_mode.update(|mode| {
            if let InlineMode::Create {
                anchor_task_id,
                depth,
                parent_id,
                ..
            } = mode
            {
                if *depth > 0 {
                    *depth -= 1;
                    let nodes = self.flat_nodes.get_untracked();
                    *parent_id = find_parent_for_depth(&nodes, *anchor_task_id, *depth);
                }
            }
        });
    }

    pub fn close_inline(&self) {
        self.inline_mode.set(InlineMode::None);
    }

    // ── Task click / detail modal ──────────────────────────────

    pub fn open_detail(&self) {
        if let Some(id) = self.cursor_task_id.get_untracked() {
            self.open_detail_for(id);
        }
    }

    pub fn open_detail_for(&self, task_id: i64) {
        if let Some(cb) = self.on_task_click {
            cb.run(task_id);
        }
    }

    #[allow(dead_code)]
    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.on_reorder.run((task_id, sort_key, parent_id));
    }

    // ── Keyboard handler ───────────────────────────────────────

    pub fn handle_keydown(&self, ev: &web_sys::KeyboardEvent) {
        let mode = self.inline_mode.get_untracked();

        match mode {
            InlineMode::None => self.handle_keydown_normal(ev),
            InlineMode::Edit { .. } => {
                // Edit input handles its own keys; nothing here.
            }
            InlineMode::Create { .. } => {
                // Create input handles its own keys; nothing here.
            }
        }
    }

    fn handle_keydown_normal(&self, ev: &web_sys::KeyboardEvent) {
        let key = ev.key();

        match key.as_str() {
            "ArrowUp" => {
                ev.prevent_default();
                self.move_up();
            }
            "ArrowDown" => {
                ev.prevent_default();
                self.move_down();
            }
            "ArrowRight" => {
                ev.prevent_default();
                self.move_right();
            }
            "ArrowLeft" => {
                ev.prevent_default();
                self.move_left();
            }
            "Enter" => {
                if ev.ctrl_key() || ev.meta_key() {
                    ev.prevent_default();
                    self.start_create(Placement::After);
                } else if ev.shift_key() {
                    ev.prevent_default();
                    self.start_create(Placement::Before);
                } else if self.cursor_task_id.get_untracked().is_some() {
                    ev.prevent_default();
                    self.start_edit();
                }
            }
            "e" | "E" => {
                ev.prevent_default();
                self.open_detail();
            }
            "Escape" => {
                self.cursor_task_id.set(None);
            }
            _ => {}
        }
    }
}
