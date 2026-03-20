use std::collections::{BTreeMap, HashSet};

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use north_dto::CreateTask;
use north_stores::{AppStore, StatusBarVariant, TaskModel, TaskTree};

use super::tree::*;
use crate::containers::task_list_item::ItemConfig;
use crate::libs::TaskTreeView;
use crate::libs::{KeepCompletedVisible, KeepTaskVisible};

#[cfg(target_arch = "wasm32")]
fn load_collapsed_ids(key: &str) -> HashSet<i64> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten())
        .and_then(|json| serde_json::from_str::<Vec<i64>>(&json).ok())
        .map(|v| v.into_iter().collect())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_collapsed_ids(_key: &str) -> HashSet<i64> {
    HashSet::new()
}

#[cfg(target_arch = "wasm32")]
fn save_collapsed_ids(key: &str, ids: &HashSet<i64>) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let vec: Vec<i64> = ids.iter().copied().collect();
        if let Ok(json) = serde_json::to_string(&vec) {
            let _ = storage.set_item(key, &json);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_collapsed_ids(_key: &str, _ids: &HashSet<i64>) {}

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
    pub search_query: RwSignal<String>,
    pub active_tag_names: RwSignal<Vec<String>>,
    pub available_tags: Memo<Vec<(String, String)>>,
    app_store: AppStore,
    /// Shared tree — available when using TaskTreeView, lazily derived otherwise.
    pub tree: Memo<TaskTree>,
    pub collapsed_ids: RwSignal<HashSet<i64>>,
    fold_storage_key: StoredValue<Option<String>>,
    allow_create: bool,
    allow_reorder: bool,
    scoped: bool,
    default_project_id: Option<Signal<Option<i64>>>,
    default_parent_id: Option<Signal<Option<i64>>>,
    keep_visible: Option<KeepTaskVisible>,
    keep_completed: Option<KeepCompletedVisible>,
    on_task_click: Option<Callback<i64>>,
    on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
}

impl TraversableTaskListController {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_store: AppStore,
        view: Option<TaskTreeView>,
        legacy_root_task_ids: Option<Memo<Vec<i64>>>,
        show_keybindings_help: RwSignal<bool>,
        on_task_click: Option<Callback<i64>>,
        on_reorder: Callback<(i64, String, Option<Option<i64>>)>,
        allow_create: bool,
        allow_reorder: bool,
        item_config: ItemConfig,
        default_project_id: Option<Signal<Option<i64>>>,
        default_parent_id: Option<Signal<Option<i64>>>,
        _flat: bool,
        scoped: bool,
        cursor_task_id: Option<RwSignal<Option<i64>>>,
        node_filter: Option<Signal<Callback<TaskModel, bool>>>,
        fold_storage_key: Option<String>,
    ) -> Self {
        let search_query: RwSignal<String> = RwSignal::new(String::new());
        let active_tag_names: RwSignal<Vec<String>> = RwSignal::new(vec![]);

        let initial_collapsed = fold_storage_key
            .as_deref()
            .map(load_collapsed_ids)
            .unwrap_or_default();
        let collapsed_ids: RwSignal<HashSet<i64>> = RwSignal::new(initial_collapsed);
        let fold_storage_key = StoredValue::new(fold_storage_key);

        // Use shared tree from TaskStore (always available)
        let tree = app_store.tasks.task_tree;

        let (flat_nodes, available_tags) = if let Some(tv) = view {
            // ── New path: TaskTreeView-based ────────────────────
            let page_filter = tv.filter;
            let extra_visible = tv.extra_visible;

            let available_tags = Memo::new(move |_| {
                let tree = tree.get();
                let filter = page_filter.get();
                let mut tag_map = BTreeMap::<String, String>::new();

                fn collect_tags_from_tree(
                    tree: &TaskTree,
                    task_id: i64,
                    tag_map: &mut BTreeMap<String, String>,
                ) {
                    if let Some(t) = tree.get(task_id) {
                        for tag in &t.tags {
                            tag_map
                                .entry(tag.name.clone())
                                .or_insert_with(|| tag.color.clone());
                        }
                        let group = tree.children_of(Some(task_id));
                        for &child_id in group.all_ids() {
                            collect_tags_from_tree(tree, child_id, tag_map);
                        }
                    }
                }

                // Collect from root tasks that pass the filter
                let root_group = tree.children_of(None);
                for &root_id in root_group.all_ids() {
                    if let Some(task) = tree.get(root_id) {
                        if filter.run(task.clone()) {
                            collect_tags_from_tree(&tree, root_id, &mut tag_map);
                        }
                    }
                }

                active_tag_names.update(|active| {
                    let available_names: Vec<String> = tag_map.keys().cloned().collect();
                    active.retain(|name| available_names.contains(name));
                });

                tag_map.into_iter().collect::<Vec<_>>()
            });

            let flat_nodes = Memo::new(move |_| {
                let tree = tree.get();
                let filter = page_filter.get();
                let extras = extra_visible.get();
                let active_tags = active_tag_names.get();
                let query = search_query.get().trim().to_lowercase();
                let collapsed = collapsed_ids.get();

                // Step 1: Flatten with page filter + extra_visible
                let nodes = tree.flatten(|t| filter.run(t.clone()), &extras);

                // Step 2: Apply local filters (tags + search) with ancestor preservation
                let nodes = if active_tags.is_empty() && query.is_empty() {
                    nodes
                } else {
                    let mut local_visible: HashSet<i64> = HashSet::new();
                    for node in &nodes {
                        if let Some(task) = tree.get(node.task_id) {
                            if !active_tags.is_empty() {
                                let tag_names: Vec<&str> =
                                    task.tags.iter().map(|tag| tag.name.as_str()).collect();
                                if !active_tags
                                    .iter()
                                    .all(|req| tag_names.contains(&req.as_str()))
                                {
                                    continue;
                                }
                            }
                            if !query.is_empty()
                                && !task.title.to_lowercase().contains(&query)
                                && !task
                                    .body
                                    .as_ref()
                                    .is_some_and(|b| b.to_lowercase().contains(&query))
                            {
                                continue;
                            }
                            local_visible.insert(task.id);
                            for anc_id in tree.ancestors(task.id) {
                                local_visible.insert(anc_id);
                            }
                        }
                    }

                    nodes
                        .into_iter()
                        .filter(|n| local_visible.contains(&n.task_id))
                        .collect()
                };

                // Step 3: Remove descendants of collapsed tasks
                if collapsed.is_empty() {
                    return nodes;
                }
                let mut result = Vec::with_capacity(nodes.len());
                let mut skip_below: Option<u8> = None;
                for node in nodes {
                    if let Some(max_d) = skip_below {
                        if node.depth > max_d {
                            continue;
                        }
                        skip_below = None;
                    }
                    if collapsed.contains(&node.task_id) {
                        skip_below = Some(node.depth);
                    }
                    result.push(node);
                }
                result
            });

            (flat_nodes, available_tags)
        } else {
            // ── Flat-mode path (filter page) ─────────────────────
            let root_task_ids =
                legacy_root_task_ids.unwrap_or_else(|| Memo::new(|_| Vec::<i64>::new()));

            let available_tags = Memo::new(move |_| {
                let tree = tree.get();
                let roots = root_task_ids.get();
                let mut tag_map = BTreeMap::<String, String>::new();

                for &root_id in &roots {
                    if let Some(t) = tree.get(root_id) {
                        for tag in &t.tags {
                            tag_map
                                .entry(tag.name.clone())
                                .or_insert_with(|| tag.color.clone());
                        }
                    }
                }

                active_tag_names.update(|active| {
                    let available_names: Vec<String> = tag_map.keys().cloned().collect();
                    active.retain(|name| available_names.contains(name));
                });

                tag_map.into_iter().collect::<Vec<_>>()
            });

            let flat_nodes = Memo::new(move |_| {
                let filter = node_filter.map(|s| s.get());
                let roots = root_task_ids.get();
                let tree = tree.get();
                let active_tags = active_tag_names.get();
                let query = search_query.get().trim().to_lowercase();

                let mut nodes = tree.flatten_flat(&roots);

                // Apply node filter (e.g. completed toggle from task detail modal)
                if let Some(ref f) = filter {
                    nodes.retain(|n| {
                        tree.get(n.task_id)
                            .map(|t| f.run(t.clone()))
                            .unwrap_or(false)
                    });
                }

                // Apply local filters (tags + search)
                if !active_tags.is_empty() || !query.is_empty() {
                    nodes.retain(|n| {
                        if let Some(task) = tree.get(n.task_id) {
                            if !active_tags.is_empty() {
                                let tag_names: Vec<&str> =
                                    task.tags.iter().map(|tag| tag.name.as_str()).collect();
                                if !active_tags
                                    .iter()
                                    .all(|req| tag_names.contains(&req.as_str()))
                                {
                                    return false;
                                }
                            }
                            if !query.is_empty()
                                && !task.title.to_lowercase().contains(&query)
                                && !task
                                    .body
                                    .as_ref()
                                    .is_some_and(|b| b.to_lowercase().contains(&query))
                            {
                                return false;
                            }
                            true
                        } else {
                            false
                        }
                    });
                }

                // Remove descendants of collapsed tasks
                let collapsed = collapsed_ids.get();
                if !collapsed.is_empty() {
                    let mut result = Vec::with_capacity(nodes.len());
                    let mut skip_below: Option<u8> = None;
                    for node in nodes {
                        if let Some(max_d) = skip_below {
                            if node.depth > max_d {
                                continue;
                            }
                            skip_below = None;
                        }
                        if collapsed.contains(&node.task_id) {
                            skip_below = Some(node.depth);
                        }
                        result.push(node);
                    }
                    return result;
                }

                nodes
            });

            (flat_nodes, available_tags)
        };

        let cursor_task_id = cursor_task_id.unwrap_or_else(|| RwSignal::new(None::<i64>));

        let cursor_index = Memo::new(move |_| {
            let id = cursor_task_id.get()?;
            let nodes = flat_nodes.get();
            nodes.iter().position(|n| n.task_id == id)
        });

        // Clear cursor when the selected task is no longer in the visible list
        Effect::new(move |_| {
            if cursor_index.get().is_none() {
                cursor_task_id.set(None);
            }
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
            search_query,
            active_tag_names,
            available_tags,
            app_store,
            tree,
            collapsed_ids,
            fold_storage_key,
            allow_create,
            allow_reorder,
            scoped,
            default_project_id,
            default_parent_id,
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
                self.cursor_task_id.set(Some(first.task_id));
            }
        } else if let Some(first) = nodes.first() {
            self.cursor_task_id.set(Some(first.task_id));
        }
    }

    pub fn move_right(&self) {
        if let Some(id) = self.cursor_task_id.get_untracked() {
            // If task is collapsed, unfold it and navigate to first child
            if self.collapsed_ids.get_untracked().contains(&id) {
                self.toggle_fold(id);
                let tree = self.tree.get_untracked();
                let group = tree.children_of(Some(id));
                if let Some(&first) = group.all_ids().next() {
                    self.cursor_task_id.set(Some(first));
                }
                return;
            }
            let nodes = self.flat_nodes.get_untracked();
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

    pub fn toggle_fold(&self, task_id: i64) {
        let tree = self.tree.get_untracked();
        if tree.children_of(Some(task_id)).is_empty() {
            return;
        }
        self.collapsed_ids.update(|ids| {
            if !ids.remove(&task_id) {
                ids.insert(task_id);
            }
        });
        self.fold_storage_key.with_value(|key| {
            if let Some(ref k) = key {
                save_collapsed_ids(k, &self.collapsed_ids.get_untracked());
            }
        });
    }

    // ── Inline edit ────────────────────────────────────────────

    pub fn start_edit(&self) {
        if let Some(id) = self.cursor_task_id.get_untracked() {
            self.inline_mode.set(InlineMode::Edit { task_id: id });
        }
    }

    pub fn save_edit(&self, new_title: String, new_body: Option<String>) {
        if let InlineMode::Edit { task_id } = self.inline_mode.get_untracked() {
            let (cleaned_title, tags) = north_dto::extract_tag_tokens(&new_title);
            if !cleaned_title.is_empty() {
                let AppStore { tasks, .. } = self.app_store;
                tasks.update_task_with_tags(task_id, cleaned_title, new_body, tags);
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
        let tree = self.tree.get_untracked();

        let sort_key =
            compute_sort_key_from_tree(&nodes, &tree, anchor_task_id, placement, parent_id);

        let project_id = parent_id
            .and_then(|pid| tree.get(pid).and_then(|t| t.project_id))
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

        let AppStore { tasks, .. } = self.app_store;
        let inline_mode = self.inline_mode;
        let keep_visible = self.keep_visible;
        spawn_local(async move {
            if let Some(task) = tasks.create_task_async(input).await {
                if let Some(kv) = keep_visible {
                    kv.keep(task.id);
                }
                if placement == Placement::After {
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
        let tree = self.tree.get_untracked();

        let parent_id = self.default_parent_id.and_then(|s| s.get_untracked());

        let first_root_key = nodes
            .iter()
            .find(|n| n.parent_id == parent_id)
            .and_then(|n| tree.sort_key(n.task_id).map(|s| s.to_string()));
        let sort_key = north_dto::sort_key_between(None, first_root_key.as_deref());

        let project_id = parent_id
            .and_then(|pid| tree.get(pid).and_then(|t| t.project_id))
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

        let AppStore { tasks, .. } = self.app_store;
        let inline_mode = self.inline_mode;
        let keep_visible = self.keep_visible;
        spawn_local(async move {
            if let Some(task) = tasks.create_task_async(input).await {
                if let Some(kv) = keep_visible {
                    kv.keep(task.id);
                }
                blur_active_element();
                inline_mode.set(InlineMode::Create {
                    anchor_task_id: task.id,
                    placement: Placement::After,
                    parent_id,
                    depth: 0,
                });
            }
        });
    }

    pub fn close_inline(&self) {
        blur_active_element();
        self.inline_mode.set(InlineMode::None);
    }

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
        let AppStore { tasks, .. } = self.app_store;
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let is_completed = tasks
            .get_by_id(task_id)
            .get_untracked()
            .map(|t| t.completed_at.is_some())
            .unwrap_or(false);

        if !is_completed {
            if let Some(kc) = self.keep_completed {
                kc.keep(task_id);
            }
            let nodes = self.flat_nodes.get_untracked();
            let next_cursor = next_sibling(&nodes, task_id)
                .or_else(|| prev_sibling(&nodes, task_id))
                .or_else(|| {
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

        tasks.toggle_complete(task_id, is_completed);
    }

    // ── Delete with confirmation ─────────────────────────────

    pub fn request_delete(&self) {
        let AppStore {
            tasks, status_bar, ..
        } = self.app_store;
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let task = tasks.get_by_id(task_id).get_untracked();
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
        status_bar.show_message(
            format!("Delete \"{title}\"?{suffix}  Enter to confirm \u{00b7} Esc to cancel"),
            StatusBarVariant::Danger,
        );
    }

    pub fn confirm_delete(&self) {
        let AppStore {
            tasks, status_bar, ..
        } = self.app_store;
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let nodes = self.flat_nodes.get_untracked();

        let next_cursor = next_sibling(&nodes, task_id)
            .or_else(|| prev_sibling(&nodes, task_id))
            .or_else(|| parent_of(&nodes, task_id));

        self.pending_delete.set(false);
        status_bar.hide_message();
        self.cursor_task_id.set(next_cursor);
        tasks.delete_task(task_id);
    }

    pub fn cancel_delete(&self) {
        let AppStore { status_bar, .. } = self.app_store;
        self.pending_delete.set(false);
        status_bar.hide_message();
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

    fn get_sort_key(&self, task_id: i64) -> Option<String> {
        self.tree
            .get_untracked()
            .sort_key(task_id)
            .map(|s| s.to_string())
    }

    pub fn reorder_up(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let siblings = self.siblings(task_id);
        let Some(pos) = siblings.iter().position(|&id| id == task_id) else {
            return;
        };
        if pos == 0 {
            return;
        }

        let above_key = if pos >= 2 {
            self.get_sort_key(siblings[pos - 2])
        } else {
            None
        };
        let below_key = self.get_sort_key(siblings[pos - 1]);
        let new_key = north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref());
        self.reorder_task(task_id, new_key, None);
    }

    pub fn reorder_down(&self) {
        let Some(task_id) = self.cursor_task_id.get_untracked() else {
            return;
        };
        let siblings = self.siblings(task_id);
        let Some(pos) = siblings.iter().position(|&id| id == task_id) else {
            return;
        };
        if pos + 1 >= siblings.len() {
            return;
        }

        let above_key = self.get_sort_key(siblings[pos + 1]);
        let below_key = siblings.get(pos + 2).and_then(|&id| self.get_sort_key(id));
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

        let last_child_key = nodes
            .iter()
            .filter(|n| n.parent_id == Some(prev_id))
            .filter_map(|n| self.get_sort_key(n.task_id))
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

        let parent_siblings: Vec<i64> = nodes
            .iter()
            .filter(|n| n.parent_id == grandparent_id)
            .map(|n| n.task_id)
            .collect();
        let parent_pos = parent_siblings
            .iter()
            .position(|&id| id == parent_id)
            .unwrap_or(0);

        let above_key = self.get_sort_key(parent_id);
        let below_key = parent_siblings
            .get(parent_pos + 1)
            .and_then(|&id| self.get_sort_key(id));
        let new_key = north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref());
        self.reorder_task(task_id, new_key, Some(grandparent_id));
    }

    pub fn task_for_edit(&self, task_id: i64) -> (String, Option<String>) {
        let AppStore { tasks, .. } = self.app_store;
        let task = tasks.get_by_id(task_id).get_untracked();
        let mut title = task.as_ref().map(|t| t.title.clone()).unwrap_or_default();
        let body = task.as_ref().and_then(|t| t.body.clone());
        // Reconstruct #tag tokens so the user can see/edit them inline
        if let Some(t) = &task {
            for tag in &t.tags {
                title.push(' ');
                title.push('#');
                title.push_str(&tag.name);
            }
        }
        (title, body)
    }

    // ── Keyboard handler ───────────────────────────────────────

    pub fn handle_keydown(&self, ev: &web_sys::KeyboardEvent) {
        let AppStore { modal, .. } = self.app_store;
        if !self.scoped && modal.is_any_open() {
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
            InlineMode::Edit { .. } => {}
            InlineMode::Create { .. } | InlineMode::CreateTop => {}
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
                        let AppStore { tasks, .. } = self.app_store;
                        tasks.review_task(task_id);
                    }
                }
            }
            "s" | "S" => {
                ev.prevent_default();
                if let Some(task_id) = self.cursor_task_id.get_untracked() {
                    let AppStore { tasks, .. } = self.app_store;
                    tasks.toggle_someday(task_id);
                }
            }
            "z" | "Z" => {
                ev.prevent_default();
                if let Some(task_id) = self.cursor_task_id.get_untracked() {
                    self.toggle_fold(task_id);
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

/// Compute a sort_key using the TaskTree for O(1) lookups instead of scanning all_tasks.
fn compute_sort_key_from_tree(
    flat: &[FlatNode],
    tree: &TaskTree,
    anchor_task_id: i64,
    placement: Placement,
    parent_id: Option<i64>,
) -> String {
    let siblings: Vec<i64> = flat
        .iter()
        .filter(|n| n.parent_id == parent_id)
        .map(|n| n.task_id)
        .collect();

    if let Some(anchor_pos) = siblings.iter().position(|&id| id == anchor_task_id) {
        match placement {
            Placement::After => {
                let above_key = tree.sort_key(anchor_task_id).map(|s| s.to_string());
                let below_key = siblings
                    .get(anchor_pos + 1)
                    .and_then(|&id| tree.sort_key(id).map(|s| s.to_string()));
                north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref())
            }
            Placement::Before => {
                let above_key = if anchor_pos > 0 {
                    siblings
                        .get(anchor_pos - 1)
                        .and_then(|&id| tree.sort_key(id).map(|s| s.to_string()))
                } else {
                    None
                };
                let below_key = tree.sort_key(anchor_task_id).map(|s| s.to_string());
                north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref())
            }
        }
    } else {
        let last_key = siblings
            .last()
            .and_then(|&id| tree.sort_key(id).map(|s| s.to_string()));
        north_dto::sort_key_after(last_key.as_deref())
    }
}
