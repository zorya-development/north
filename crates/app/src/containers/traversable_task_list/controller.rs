use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use north_dto::CreateTask;
use north_stores::{AppStore, ModalStore, StatusBarVariant, TaskModel, TaskStoreFilter};

use super::tree::*;
use crate::containers::task_list_item::ItemConfig;
use crate::libs::{KeepCompletedVisible, KeepTaskVisible};

/// Blur the currently focused element so that blur handlers fire while
/// signals/callbacks are still alive — before a `<Show>` disposes the scope.
fn blur_active_element() {
    if let Some(el) = document().active_element() {
        if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
            let _ = html_el.blur();
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct TraversableTaskListController {
    pub flat_nodes: Memo<Vec<FlatNode>>,
    pub cursor_task_id: RwSignal<Option<i64>>,
    pub cursor_index: Memo<Option<usize>>,
    pub inline_mode: RwSignal<InlineMode>,
    pub create_input_value: RwSignal<String>,
    pub pending_delete: RwSignal<bool>,
    pub show_keybindings_help: RwSignal<bool>,
    pub item_config: ItemConfig,
    app_store: AppStore,
    modal: ModalStore,
    allow_create: bool,
    allow_reorder: bool,
    scoped: bool,
    default_project_id: Option<Signal<Option<i64>>>,
    keep_visible: Option<KeepTaskVisible>,
    keep_completed: Option<KeepCompletedVisible>,
    on_task_click: Option<Callback<i64>>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
}

impl TraversableTaskListController {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_store: AppStore,
        modal: ModalStore,
        root_task_ids: Memo<Vec<i64>>,
        show_keybindings_help: RwSignal<bool>,
        on_task_click: Option<Callback<i64>>,
        on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
        allow_create: bool,
        allow_reorder: bool,
        item_config: ItemConfig,
        default_project_id: Option<Signal<Option<i64>>>,
        flat: bool,
        scoped: bool,
        cursor_task_id: Option<RwSignal<Option<i64>>>,
        node_filter: Option<Signal<Callback<TaskModel, bool>>>,
    ) -> Self {
        let all_tasks = app_store.tasks.filtered(TaskStoreFilter::default());

        let flat_nodes = Memo::new(move |_| {
            let filter = node_filter.map(|s| s.get());
            let roots = root_task_ids.get();
            let tasks = all_tasks.get();
            let include = |t: &TaskModel| filter.as_ref().map(|f| f.run(t.clone())).unwrap_or(true);
            if flat {
                flatten_flat(&roots, &tasks, &include)
            } else {
                flatten_tree(&roots, &tasks, &include)
            }
        });

        let cursor_task_id = cursor_task_id.unwrap_or_else(|| RwSignal::new(None::<i64>));

        let cursor_index = Memo::new(move |_| {
            let id = cursor_task_id.get()?;
            let nodes = flat_nodes.get();
            nodes.iter().position(|n| n.task_id == id)
        });

        let inline_mode = RwSignal::new(InlineMode::None);
        let create_input_value = RwSignal::new(String::new());
        let pending_delete = RwSignal::new(false);
        let keep_visible = use_context::<KeepTaskVisible>();
        let keep_completed = use_context::<KeepCompletedVisible>();

        Self {
            flat_nodes,
            cursor_task_id,
            cursor_index,
            inline_mode,
            create_input_value,
            pending_delete,
            show_keybindings_help,
            item_config,
            app_store,
            modal,
            allow_create,
            allow_reorder,
            scoped,
            default_project_id,
            keep_visible,
            keep_completed,
            on_task_click,
            on_reorder,
        }
    }

    // ── Cursor navigation ──────────────────────────────────────

    pub fn move_up(&self) {
        let nodes = self.flat_nodes.get_untracked();
        if let Some(id) = self.cursor_task_id.get_untracked() {
            if let Some(idx) = nodes.iter().position(|n| n.task_id == id) {
                if idx > 0 {
                    self.cursor_task_id.set(Some(nodes[idx - 1].task_id));
                }
            } else if let Some(last) = nodes.last() {
                // Cursor task no longer in list — recover to last
                self.cursor_task_id.set(Some(last.task_id));
            }
        } else if let Some(last) = nodes.last() {
            self.cursor_task_id.set(Some(last.task_id));
        }
    }

    pub fn move_down(&self) {
        let nodes = self.flat_nodes.get_untracked();
        if let Some(id) = self.cursor_task_id.get_untracked() {
            if let Some(idx) = nodes.iter().position(|n| n.task_id == id) {
                if idx + 1 < nodes.len() {
                    self.cursor_task_id.set(Some(nodes[idx + 1].task_id));
                }
            } else if let Some(first) = nodes.first() {
                // Cursor task no longer in list — recover to first
                self.cursor_task_id.set(Some(first.task_id));
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

    pub fn save_edit(&self, new_title: String, new_body: Option<String>) {
        if let InlineMode::Edit { task_id } = self.inline_mode.get_untracked() {
            if !new_title.is_empty() {
                self.app_store
                    .tasks
                    .update_task(task_id, new_title, new_body);
            }
            blur_active_element();
            self.inline_mode.set(InlineMode::None);
        }
    }

    pub fn cancel_edit(&self) {
        if matches!(self.inline_mode.get_untracked(), InlineMode::Edit { .. }) {
            blur_active_element();
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

    pub fn start_create_top(&self) {
        self.create_input_value.set(String::new());
        self.inline_mode.set(InlineMode::CreateTop);
    }

    pub fn start_create_inside(&self) {
        if let Some(anchor_id) = self.cursor_task_id.get_untracked() {
            let nodes = self.flat_nodes.get_untracked();
            if let Some(anchor) = nodes.iter().find(|n| n.task_id == anchor_id) {
                self.create_input_value.set(String::new());
                self.inline_mode.set(InlineMode::Create {
                    anchor_task_id: anchor_id,
                    placement: Placement::After,
                    parent_id: Some(anchor_id),
                    depth: anchor.depth + 1,
                });
            }
        }
    }

    pub fn create_task(&self) {
        let mode = self.inline_mode.get_untracked();

        match mode {
            InlineMode::Create {
                anchor_task_id,
                placement,
                parent_id,
                depth,
            } => {
                self.create_task_anchored(anchor_task_id, placement, parent_id, depth);
            }
            InlineMode::CreateTop => {
                self.create_task_top();
            }
            _ => {}
        }
    }

    fn create_task_anchored(
        &self,
        anchor_task_id: i64,
        placement: Placement,
        parent_id: Option<i64>,
        depth: u8,
    ) {
        let (title, body) = Self::parse_title_body(&self.create_input_value.get_untracked());
        if title.is_empty() {
            self.close_inline();
            return;
        }

        let nodes = self.flat_nodes.get_untracked();
        let all_tasks = self.app_store.tasks.filtered(TaskStoreFilter::default());
        let tasks = all_tasks.get_untracked();

        let sort_key = compute_sort_key(&nodes, &tasks, anchor_task_id, placement, parent_id);

        let project_id = parent_id
            .and_then(|pid| {
                tasks
                    .iter()
                    .find(|t| t.id == pid)
                    .and_then(|t| t.project_id)
            })
            .or_else(|| self.default_project_id.and_then(|s| s.get_untracked()));

        let input = CreateTask {
            title,
            body,
            parent_id,
            project_id,
            sort_key: Some(sort_key),
            ..Default::default()
        };

        self.create_input_value.set(String::new());

        let store = self.app_store.tasks;
        let inline_mode = self.inline_mode;
        let keep_visible = self.keep_visible;
        spawn_local(async move {
            if let Some(task) = store.create_task_async(input).await {
                if let Some(kv) = keep_visible {
                    kv.keep(task.id);
                }
                // For After placement, chain: next create goes after the
                // newly created task. For Before, anchor stays the same.
                if placement == Placement::After {
                    // Blur before changing inline_mode to prevent disposed-callback
                    // panics: the old <Show> scope will tear down, and the browser
                    // fires blur into handlers whose Callbacks are already disposed.
                    blur_active_element();
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

    fn create_task_top(&self) {
        let (title, body) = Self::parse_title_body(&self.create_input_value.get_untracked());
        if title.is_empty() {
            self.close_inline();
            return;
        }

        let nodes = self.flat_nodes.get_untracked();
        let all_tasks = self.app_store.tasks.filtered(TaskStoreFilter::default());
        let tasks = all_tasks.get_untracked();

        // Sort key: before the first root task.
        let first_root_key = nodes
            .iter()
            .find(|n| n.parent_id.is_none())
            .and_then(|n| task_sort_key(&tasks, n.task_id));
        let sort_key = north_dto::sort_key_between(None, first_root_key.as_deref());

        let project_id = self.default_project_id.and_then(|s| s.get_untracked());

        let input = CreateTask {
            title,
            body,
            parent_id: None,
            project_id,
            sort_key: Some(sort_key),
            ..Default::default()
        };

        self.create_input_value.set(String::new());

        let store = self.app_store.tasks;
        let inline_mode = self.inline_mode;
        let keep_visible = self.keep_visible;
        spawn_local(async move {
            if let Some(task) = store.create_task_async(input).await {
                if let Some(kv) = keep_visible {
                    kv.keep(task.id);
                }
                // Blur before changing inline_mode (see create_task_anchored).
                blur_active_element();
                // Chain: next create goes after the newly created task.
                inline_mode.set(InlineMode::Create {
                    anchor_task_id: task.id,
                    placement: Placement::After,
                    parent_id: None,
                    depth: 0,
                });
            }
        });
    }

    pub fn close_inline(&self) {
        blur_active_element();
        self.inline_mode.set(InlineMode::None);
    }

    /// Split raw input into (title, optional body).
    /// First line becomes the title; remaining lines become the body.
    fn parse_title_body(raw: &str) -> (String, Option<String>) {
        let mut lines = raw.splitn(2, '\n');
        let title = lines.next().unwrap_or("").trim().to_string();
        let body = lines
            .next()
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty());
        (title, body)
    }

    // ── Toggle complete ────────────────────────────────────────

    pub fn toggle_complete(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let is_completed = self
            .app_store
            .tasks
            .get_by_id(task_id)
            .get_untracked()
            .map(|t| t.completed_at.is_some())
            .unwrap_or(false);

        // When completing, pin the task so it stays visible until refresh,
        // then advance cursor to neighbor before the task moves to the
        // completed group.
        if !is_completed {
            if let Some(kc) = self.keep_completed {
                kc.keep(task_id);
            }
            let nodes = self.flat_nodes.get_untracked();
            let next_cursor = next_sibling(&nodes, task_id)
                .or_else(|| prev_sibling(&nodes, task_id))
                .or_else(|| {
                    // No siblings left — pick the next item in flat order so
                    // the cursor stays at the same visual position rather than
                    // jumping up to the parent.
                    let idx = nodes.iter().position(|n| n.task_id == task_id)?;
                    if idx + 1 < nodes.len() {
                        Some(nodes[idx + 1].task_id)
                    } else if idx > 0 {
                        Some(nodes[idx - 1].task_id)
                    } else {
                        None
                    }
                });
            self.cursor_task_id.set(next_cursor);
        }

        self.app_store.tasks.toggle_complete(task_id, is_completed);
    }

    // ── Delete with confirmation ─────────────────────────────

    pub fn request_delete(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let task = self.app_store.tasks.get_by_id(task_id).get_untracked();
        let title = task.as_ref().map(|t| t.title.clone()).unwrap_or_default();
        let has_recurrence = task
            .as_ref()
            .map(|t| t.recurrence.is_some())
            .unwrap_or(false);
        self.pending_delete.set(true);
        let suffix = if has_recurrence {
            " Recurring subtasks will stop."
        } else {
            ""
        };
        self.app_store.status_bar.show_message(
            format!("Delete \"{title}\"?{suffix}  Enter to confirm \u{00b7} Esc to cancel"),
            StatusBarVariant::Danger,
        );
    }

    pub fn confirm_delete(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let nodes = self.flat_nodes.get_untracked();

        // Next sibling → prev sibling → parent → None.
        let next_cursor = next_sibling(&nodes, task_id)
            .or_else(|| prev_sibling(&nodes, task_id))
            .or_else(|| parent_of(&nodes, task_id));

        self.pending_delete.set(false);
        self.app_store.status_bar.hide_message();
        self.cursor_task_id.set(next_cursor);
        self.app_store.tasks.delete_task(task_id);
    }

    pub fn cancel_delete(&self) {
        self.pending_delete.set(false);
        self.app_store.status_bar.hide_message();
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

    pub fn reorder_task(&self, task_id: i64, sort_key: String, parent_id: Option<Option<i64>>) {
        self.on_reorder.run((task_id, sort_key, parent_id));
    }

    // ── Task reorder (Shift+Arrow) ──────────────────────────────

    fn all_tasks(&self) -> Vec<TaskModel> {
        self.app_store
            .tasks
            .filtered(TaskStoreFilter::default())
            .get_untracked()
    }

    fn siblings(&self, task_id: i64) -> Vec<i64> {
        let nodes = self.flat_nodes.get_untracked();
        let parent_id = nodes
            .iter()
            .find(|n| n.task_id == task_id)
            .map(|n| n.parent_id);
        let Some(parent_id) = parent_id else {
            return vec![];
        };
        nodes
            .iter()
            .filter(|n| n.parent_id == parent_id)
            .map(|n| n.task_id)
            .collect()
    }

    pub fn reorder_up(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let siblings = self.siblings(task_id);
        let tasks = self.all_tasks();
        let Some(pos) = siblings.iter().position(|&id| id == task_id) else {
            return;
        };
        if pos == 0 {
            return;
        }

        let above_key = if pos >= 2 {
            task_sort_key(&tasks, siblings[pos - 2])
        } else {
            None
        };
        let below_key = task_sort_key(&tasks, siblings[pos - 1]);
        let new_key = north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref());
        self.reorder_task(task_id, new_key, None);
    }

    pub fn reorder_down(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let siblings = self.siblings(task_id);
        let tasks = self.all_tasks();
        let Some(pos) = siblings.iter().position(|&id| id == task_id) else {
            return;
        };
        if pos + 1 >= siblings.len() {
            return;
        }

        let above_key = task_sort_key(&tasks, siblings[pos + 1]);
        let below_key = siblings
            .get(pos + 2)
            .and_then(|&id| task_sort_key(&tasks, id));
        let new_key = north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref());
        self.reorder_task(task_id, new_key, None);
    }

    pub fn reorder_right(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let nodes = self.flat_nodes.get_untracked();
        let Some(prev_id) = prev_sibling(&nodes, task_id) else {
            return;
        };
        let tasks = self.all_tasks();

        // Become last child of previous sibling.
        let last_child_key = nodes
            .iter()
            .filter(|n| n.parent_id == Some(prev_id))
            .filter_map(|n| task_sort_key(&tasks, n.task_id))
            .next_back();
        let new_key = north_dto::sort_key_after(last_child_key.as_deref());
        self.reorder_task(task_id, new_key, Some(Some(prev_id)));
    }

    pub fn reorder_left(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let nodes = self.flat_nodes.get_untracked();
        let node = nodes.iter().find(|n| n.task_id == task_id);
        let Some(parent_id) = node.and_then(|n| n.parent_id) else {
            return;
        };

        let parent_node = nodes.iter().find(|n| n.task_id == parent_id);
        let grandparent_id = parent_node.and_then(|n| n.parent_id);
        let tasks = self.all_tasks();

        // Place after parent among grandparent's children.
        let parent_siblings: Vec<i64> = nodes
            .iter()
            .filter(|n| n.parent_id == grandparent_id)
            .map(|n| n.task_id)
            .collect();
        let parent_pos = parent_siblings
            .iter()
            .position(|&id| id == parent_id)
            .unwrap_or(0);

        let above_key = task_sort_key(&tasks, parent_id);
        let below_key = parent_siblings
            .get(parent_pos + 1)
            .and_then(|&id| task_sort_key(&tasks, id));
        let new_key = north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref());
        self.reorder_task(task_id, new_key, Some(grandparent_id));
    }

    // ── Keyboard handler ───────────────────────────────────────

    pub fn handle_keydown(&self, ev: &web_sys::KeyboardEvent) {
        if !self.scoped && self.modal.is_any_open() {
            return;
        }

        if self.show_keybindings_help.get_untracked() {
            if ev.key() == "Escape" {
                ev.prevent_default();
                self.show_keybindings_help.set(false);
            }
            return;
        }

        if self.pending_delete.get_untracked() {
            match ev.key().as_str() {
                "Enter" => {
                    ev.prevent_default();
                    self.confirm_delete();
                }
                _ => {
                    ev.prevent_default();
                    self.cancel_delete();
                }
            }
            return;
        }

        let mode = self.inline_mode.get_untracked();

        match mode {
            InlineMode::None => self.handle_keydown_normal(ev),
            InlineMode::Edit { .. } => {
                // Edit input handles its own keys; nothing here.
            }
            InlineMode::Create { .. } | InlineMode::CreateTop => {
                // Create input handles its own keys; nothing here.
            }
        }
    }

    fn handle_keydown_normal(&self, ev: &web_sys::KeyboardEvent) {
        let key = ev.key();

        match key.as_str() {
            "ArrowUp" => {
                ev.prevent_default();
                if ev.shift_key() && self.allow_reorder {
                    self.reorder_up();
                } else if !ev.shift_key() {
                    self.move_up();
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                if ev.shift_key() && self.allow_reorder {
                    self.reorder_down();
                } else if !ev.shift_key() {
                    self.move_down();
                }
            }
            "ArrowRight" => {
                ev.prevent_default();
                if ev.shift_key() && self.allow_reorder {
                    self.reorder_right();
                } else if !ev.shift_key() {
                    self.move_right();
                }
            }
            "ArrowLeft" => {
                ev.prevent_default();
                if ev.shift_key() && self.allow_reorder {
                    self.reorder_left();
                } else if !ev.shift_key() {
                    self.move_left();
                }
            }
            "Enter" => {
                if (ev.ctrl_key() || ev.meta_key()) && ev.shift_key() {
                    if self.allow_create {
                        ev.prevent_default();
                        self.start_create_inside();
                    }
                } else if ev.ctrl_key() || ev.meta_key() {
                    if self.allow_create {
                        ev.prevent_default();
                        self.start_create(Placement::After);
                    }
                } else if ev.shift_key() {
                    if self.allow_create {
                        ev.prevent_default();
                        self.start_create(Placement::Before);
                    }
                } else if self.cursor_task_id.get_untracked().is_some() {
                    ev.prevent_default();
                    self.start_edit();
                }
            }
            "e" | "E" => {
                ev.prevent_default();
                self.open_detail();
            }
            "r" | "R" => {
                if self.item_config.show_review {
                    ev.prevent_default();
                    if let Some(task_id) = self.cursor_task_id.get_untracked() {
                        self.app_store.tasks.review_task(task_id);
                    }
                }
            }
            "s" | "S" => {
                ev.prevent_default();
                if let Some(task_id) = self.cursor_task_id.get_untracked() {
                    self.app_store.tasks.toggle_someday(task_id);
                }
            }
            " " => {
                ev.prevent_default();
                self.toggle_complete();
            }
            "Delete" => {
                ev.prevent_default();
                self.request_delete();
            }
            "Escape" => {
                self.cursor_task_id.set(None);
            }
            "?" => {
                ev.prevent_default();
                self.show_keybindings_help.set(true);
            }
            _ => {}
        }
    }
}
