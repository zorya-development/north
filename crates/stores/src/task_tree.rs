use std::collections::{HashMap, HashSet};

use north_repositories::TaskModel;

/// A flattened tree node for list rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct FlatNode {
    pub task_id: i64,
    pub parent_id: Option<i64>,
    pub depth: u8,
    pub is_completed: bool,
    pub is_someday: bool,
}

/// Pre-sorted children of a task, grouped by status.
#[derive(Clone, Debug, Default)]
pub struct ChildGroup {
    /// Not completed, not someday — sorted by sort_key.
    pub active: Vec<i64>,
    /// Not completed, is someday — sorted by sort_key.
    pub someday: Vec<i64>,
    /// Completed — sorted by sort_key.
    pub completed: Vec<i64>,
}

impl ChildGroup {
    /// Iterate all child IDs in display order: active → someday → completed.
    pub fn all_ids(&self) -> impl Iterator<Item = &i64> {
        self.active
            .iter()
            .chain(self.someday.iter())
            .chain(self.completed.iter())
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty() && self.someday.is_empty() && self.completed.is_empty()
    }
}

/// Indexed task tree with O(1) lookups and pre-sorted children.
///
/// Built from a flat `Vec<TaskModel>`, this structures tasks into an indexed
/// tree with parent→children relationships and domain methods for actionable
/// computation, ancestry checks, and filtered flattening.
#[derive(Clone, Debug)]
pub struct TaskTree {
    by_id: HashMap<i64, TaskModel>,
    children: HashMap<Option<i64>, ChildGroup>,
}

impl PartialEq for TaskTree {
    fn eq(&self, _: &Self) -> bool {
        false // Always notify — tree rebuild means dependents recompute
    }
}

impl Default for TaskTree {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskTree {
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            children: HashMap::new(),
        }
    }

    /// Build an indexed tree from a flat task list.
    pub fn build(tasks: Vec<TaskModel>) -> Self {
        let by_id: HashMap<i64, TaskModel> = tasks.into_iter().map(|t| (t.id, t)).collect();

        let mut parent_children: HashMap<Option<i64>, Vec<i64>> = HashMap::new();
        for (&id, task) in &by_id {
            parent_children.entry(task.parent_id).or_default().push(id);
        }

        let mut children: HashMap<Option<i64>, ChildGroup> = HashMap::new();
        for (parent_id, child_ids) in parent_children {
            let mut active = Vec::new();
            let mut someday = Vec::new();
            let mut completed = Vec::new();

            for id in child_ids {
                let task = &by_id[&id];
                if task.completed_at.is_some() {
                    completed.push(id);
                } else if task.someday {
                    someday.push(id);
                } else {
                    active.push(id);
                }
            }

            active.sort_by(|a, b| by_id[a].sort_key.cmp(&by_id[b].sort_key));
            someday.sort_by(|a, b| by_id[a].sort_key.cmp(&by_id[b].sort_key));
            completed.sort_by(|a, b| by_id[a].sort_key.cmp(&by_id[b].sort_key));

            children.insert(
                parent_id,
                ChildGroup {
                    active,
                    someday,
                    completed,
                },
            );
        }

        Self { by_id, children }
    }

    // ── Lookups ────────────────────────────────────────────────

    /// O(1) task lookup by ID.
    pub fn get(&self, id: i64) -> Option<&TaskModel> {
        self.by_id.get(&id)
    }

    /// O(1) children lookup. Returns empty group if no children exist.
    pub fn children_of(&self, parent_id: Option<i64>) -> &ChildGroup {
        static EMPTY: ChildGroup = ChildGroup {
            active: Vec::new(),
            someday: Vec::new(),
            completed: Vec::new(),
        };
        self.children.get(&parent_id).unwrap_or(&EMPTY)
    }

    /// O(1) parent lookup.
    pub fn parent_of(&self, id: i64) -> Option<i64> {
        self.get(id).and_then(|t| t.parent_id)
    }

    /// O(1) sort key lookup.
    pub fn sort_key(&self, task_id: i64) -> Option<&str> {
        self.get(task_id).map(|t| t.sort_key.as_str())
    }

    /// Walk parent chain from task to root.
    /// Returns `[parent, grandparent, ..., root]`. Does NOT include the task itself.
    pub fn ancestors(&self, id: i64) -> Vec<i64> {
        let mut result = Vec::new();
        let mut current = id;
        for _ in 0..100 {
            let Some(task) = self.get(current) else { break };
            let Some(parent_id) = task.parent_id else {
                break;
            };
            result.push(parent_id);
            current = parent_id;
        }
        result
    }

    /// Check if `task_id` is a descendant of `ancestor_id`. O(depth).
    pub fn is_descendant(&self, ancestor_id: i64, task_id: i64) -> bool {
        if ancestor_id == task_id {
            return false;
        }
        let mut current = task_id;
        for _ in 0..100 {
            let Some(task) = self.get(current) else {
                return false;
            };
            let Some(parent_id) = task.parent_id else {
                return false;
            };
            if parent_id == ancestor_id {
                return true;
            }
            current = parent_id;
        }
        false
    }

    // ── Domain methods ─────────────────────────────────────────

    /// Check if a task is actionable based on parent's sequential_limit.
    ///
    /// - Root tasks are always actionable.
    /// - Someday tasks are never actionable.
    /// - Subtasks are actionable if they are within the first N incomplete
    ///   siblings (sorted by sort_key) where N is the parent's sequential_limit.
    ///   A limit of 0 means unlimited.
    pub fn is_actionable(&self, task_id: i64) -> bool {
        let Some(task) = self.get(task_id) else {
            return false;
        };
        if task.someday {
            return false;
        }
        let Some(parent_id) = task.parent_id else {
            return true;
        };
        let limit = self.get(parent_id).map(|p| p.sequential_limit).unwrap_or(1);
        if limit == 0 {
            return true;
        }

        let group = self.children_of(Some(parent_id));
        let task_sort_key = task.sort_key.as_str();

        // Count incomplete siblings (active + someday) with sort_key < this task's sort_key
        let count: usize = group
            .active
            .iter()
            .chain(group.someday.iter())
            .filter(|&&sib_id| {
                self.get(sib_id)
                    .map(|s| s.sort_key.as_str() < task_sort_key)
                    .unwrap_or(false)
            })
            .count();

        (count as i16) < limit
    }

    /// Count tasks matching an arbitrary predicate.
    pub fn count_matching(&self, pred: impl Fn(&TaskModel) -> bool) -> usize {
        self.by_id.values().filter(|t| pred(t)).count()
    }

    /// Iterate all tasks (no guaranteed order).
    pub fn tasks(&self) -> impl Iterator<Item = &TaskModel> {
        self.by_id.values()
    }

    // ── Flattening ─────────────────────────────────────────────

    /// Flatten the tree into display order with ancestor preservation.
    ///
    /// - Root tasks must independently match the filter (or be in extra_visible).
    /// - Within a matched root's subtree, matching descendants have their
    ///   ancestor path preserved (shown even if intermediate nodes fail the filter).
    /// - `extra_visible` IDs bypass the filter entirely.
    ///
    /// Display order per parent group: active → someday → completed.
    pub fn flatten(
        &self,
        filter: impl Fn(&TaskModel) -> bool,
        extra_visible: &[i64],
    ) -> Vec<FlatNode> {
        let extra_set: HashSet<i64> = extra_visible.iter().copied().collect();
        let mut visible: HashSet<i64> = HashSet::new();

        // Phase 1: Make extra_visible tasks and their ancestors visible
        for &id in extra_visible {
            if self.get(id).is_some() {
                visible.insert(id);
                for anc_id in self.ancestors(id) {
                    visible.insert(anc_id);
                }
            }
        }

        // Phase 2: Find root tasks that pass the filter
        let root_group = self.children_of(None);
        for id in root_group.all_ids() {
            if !visible.contains(id) {
                if let Some(task) = self.get(*id) {
                    if filter(task) {
                        visible.insert(*id);
                    }
                }
            }
        }

        // Phase 3: For each visible root, scan subtree for matching descendants
        let visible_roots: Vec<i64> = root_group
            .all_ids()
            .copied()
            .filter(|id| visible.contains(id))
            .collect();

        for &root_id in &visible_roots {
            self.collect_visible_descendants(root_id, &filter, &extra_set, &mut visible);
        }

        // Phase 4: DFS from visible roots in display order
        let mut nodes = Vec::new();
        for id in root_group.all_ids() {
            if visible.contains(id) {
                self.dfs_flatten(*id, 0, &visible, &mut nodes);
            }
        }

        nodes
    }

    /// Flatten as a flat list (no tree expansion). Used by filter page.
    /// Preserves the order of `ids`.
    pub fn flatten_flat(&self, ids: &[i64]) -> Vec<FlatNode> {
        ids.iter()
            .filter_map(|&id| self.get(id))
            .map(|t| FlatNode {
                task_id: t.id,
                parent_id: t.parent_id,
                depth: 0,
                is_completed: t.completed_at.is_some(),
                is_someday: t.someday,
            })
            .collect()
    }

    // ── Internal helpers ───────────────────────────────────────

    fn collect_visible_descendants(
        &self,
        parent_id: i64,
        filter: &impl Fn(&TaskModel) -> bool,
        extra_set: &HashSet<i64>,
        visible: &mut HashSet<i64>,
    ) {
        let group = self.children_of(Some(parent_id));
        for id in group.all_ids() {
            if let Some(task) = self.get(*id) {
                if filter(task) || extra_set.contains(id) {
                    visible.insert(*id);
                    for anc_id in self.ancestors(*id) {
                        visible.insert(anc_id);
                    }
                }
            }
            self.collect_visible_descendants(*id, filter, extra_set, visible);
        }
    }

    fn dfs_flatten(&self, id: i64, depth: u8, visible: &HashSet<i64>, nodes: &mut Vec<FlatNode>) {
        let Some(task) = self.get(id) else { return };

        nodes.push(FlatNode {
            task_id: id,
            parent_id: task.parent_id,
            depth,
            is_completed: task.completed_at.is_some(),
            is_someday: task.someday,
        });

        let group = self.children_of(Some(id));
        for child_id in group.all_ids() {
            if visible.contains(child_id) {
                self.dfs_flatten(*child_id, depth + 1, visible, nodes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_task(id: i64, parent_id: Option<i64>, sort_key: &str) -> TaskModel {
        TaskModel {
            id,
            project_id: None,
            parent_id,
            user_id: 1,
            title: format!("Task {id}"),
            body: None,
            sort_key: sort_key.to_string(),
            sequential_limit: 0,
            start_at: None,
            due_date: None,
            completed_at: None,
            reviewed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            recurrence: None,
            is_url_fetching: None,
            someday: false,
            project_title: None,
            tags: vec![],
            subtask_count: 0,
            completed_subtask_count: 0,
        }
    }

    fn make_completed_task(id: i64, parent_id: Option<i64>, sort_key: &str) -> TaskModel {
        let mut t = make_task(id, parent_id, sort_key);
        t.completed_at = Some(Utc::now());
        t
    }

    fn make_someday_task(id: i64, parent_id: Option<i64>, sort_key: &str) -> TaskModel {
        let mut t = make_task(id, parent_id, sort_key);
        t.someday = true;
        t
    }

    fn task_ids(nodes: &[FlatNode]) -> Vec<i64> {
        nodes.iter().map(|n| n.task_id).collect()
    }

    // ── Build & Lookups ────────────────────────────────────────

    #[test]
    fn build_indexes_by_id() {
        let tree = TaskTree::build(vec![make_task(1, None, "a"), make_task(2, None, "b")]);
        assert!(tree.get(1).is_some());
        assert!(tree.get(2).is_some());
        assert!(tree.get(99).is_none());
    }

    #[test]
    fn children_of_groups_correctly() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_task(2, Some(1), "a"),
            make_completed_task(3, Some(1), "b"),
            make_someday_task(4, Some(1), "c"),
        ]);
        let group = tree.children_of(Some(1));
        assert_eq!(group.active, vec![2]);
        assert_eq!(group.someday, vec![4]);
        assert_eq!(group.completed, vec![3]);
    }

    #[test]
    fn children_sorted_by_sort_key() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_task(3, Some(1), "c"),
            make_task(2, Some(1), "a"),
            make_task(4, Some(1), "b"),
        ]);
        let group = tree.children_of(Some(1));
        assert_eq!(group.active, vec![2, 4, 3]);
    }

    #[test]
    fn children_of_missing_parent_returns_empty() {
        let tree = TaskTree::build(vec![make_task(1, None, "a")]);
        let group = tree.children_of(Some(99));
        assert!(group.is_empty());
    }

    #[test]
    fn parent_of_root_is_none() {
        let tree = TaskTree::build(vec![make_task(1, None, "a")]);
        assert_eq!(tree.parent_of(1), None);
    }

    #[test]
    fn parent_of_child() {
        let tree = TaskTree::build(vec![make_task(1, None, "a"), make_task(2, Some(1), "a")]);
        assert_eq!(tree.parent_of(2), Some(1));
    }

    #[test]
    fn sort_key_lookup() {
        let tree = TaskTree::build(vec![make_task(1, None, "abc")]);
        assert_eq!(tree.sort_key(1), Some("abc"));
        assert_eq!(tree.sort_key(99), None);
    }

    // ── Ancestors ──────────────────────────────────────────────

    #[test]
    fn ancestors_of_root_is_empty() {
        let tree = TaskTree::build(vec![make_task(1, None, "a")]);
        assert!(tree.ancestors(1).is_empty());
    }

    #[test]
    fn ancestors_of_child() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_task(2, Some(1), "a"),
            make_task(3, Some(2), "a"),
        ]);
        assert_eq!(tree.ancestors(3), vec![2, 1]);
    }

    // ── is_descendant ──────────────────────────────────────────

    #[test]
    fn is_descendant_direct_child() {
        let tree = TaskTree::build(vec![make_task(1, None, "a"), make_task(2, Some(1), "a")]);
        assert!(tree.is_descendant(1, 2));
        assert!(!tree.is_descendant(2, 1));
    }

    #[test]
    fn is_descendant_grandchild() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_task(2, Some(1), "a"),
            make_task(3, Some(2), "a"),
        ]);
        assert!(tree.is_descendant(1, 3));
        assert!(!tree.is_descendant(3, 1));
    }

    #[test]
    fn is_descendant_unrelated() {
        let tree = TaskTree::build(vec![make_task(1, None, "a"), make_task(2, None, "b")]);
        assert!(!tree.is_descendant(1, 2));
        assert!(!tree.is_descendant(2, 1));
    }

    #[test]
    fn is_descendant_self() {
        let tree = TaskTree::build(vec![make_task(1, None, "a")]);
        assert!(!tree.is_descendant(1, 1));
    }

    // ── is_actionable ──────────────────────────────────────────

    #[test]
    fn root_is_always_actionable() {
        let tree = TaskTree::build(vec![make_task(1, None, "a")]);
        assert!(tree.is_actionable(1));
    }

    #[test]
    fn someday_is_never_actionable() {
        let tree = TaskTree::build(vec![make_someday_task(1, None, "a")]);
        assert!(!tree.is_actionable(1));
    }

    #[test]
    fn sequential_limit_zero_means_unlimited() {
        let mut parent = make_task(1, None, "a");
        parent.sequential_limit = 0;
        let tree = TaskTree::build(vec![
            parent,
            make_task(2, Some(1), "a"),
            make_task(3, Some(1), "b"),
            make_task(4, Some(1), "c"),
        ]);
        assert!(tree.is_actionable(2));
        assert!(tree.is_actionable(3));
        assert!(tree.is_actionable(4));
    }

    #[test]
    fn sequential_limit_restricts() {
        let mut parent = make_task(1, None, "a");
        parent.sequential_limit = 2;
        let tree = TaskTree::build(vec![
            parent,
            make_task(2, Some(1), "a"),
            make_task(3, Some(1), "b"),
            make_task(4, Some(1), "c"),
        ]);
        assert!(tree.is_actionable(2)); // 0 before → < 2
        assert!(tree.is_actionable(3)); // 1 before → < 2
        assert!(!tree.is_actionable(4)); // 2 before → not < 2
    }

    #[test]
    fn sequential_limit_counts_someday_siblings() {
        let mut parent = make_task(1, None, "a");
        parent.sequential_limit = 1;
        let tree = TaskTree::build(vec![
            parent,
            make_someday_task(2, Some(1), "a"), // someday, sort_key "a"
            make_task(3, Some(1), "b"),         // active, sort_key "b"
        ]);
        // Task 3 has 1 incomplete sibling (task 2, someday) with sort_key < "b"
        // So siblings_before = 1, limit = 1, 1 < 1 = false
        assert!(!tree.is_actionable(3));
    }

    // ── count_matching ─────────────────────────────────────────

    #[test]
    fn count_matching_basic() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_completed_task(2, None, "b"),
            make_task(3, None, "c"),
        ]);
        assert_eq!(tree.count_matching(|t| t.completed_at.is_some()), 1);
        assert_eq!(tree.count_matching(|t| t.completed_at.is_none()), 2);
    }

    // ── Flatten ────────────────────────────────────────────────

    #[test]
    fn flatten_basic() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_task(2, None, "b"),
            make_task(3, Some(1), "a"),
            make_task(4, Some(1), "b"),
        ]);
        let nodes = tree.flatten(|_| true, &[]);
        assert_eq!(task_ids(&nodes), vec![1, 3, 4, 2]);
        assert_eq!(nodes[0].depth, 0);
        assert_eq!(nodes[1].depth, 1);
        assert_eq!(nodes[2].depth, 1);
        assert_eq!(nodes[3].depth, 0);
    }

    #[test]
    fn flatten_hides_completed() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_completed_task(2, None, "b"),
        ]);
        let nodes = tree.flatten(|t| t.completed_at.is_none(), &[]);
        assert_eq!(task_ids(&nodes), vec![1]);
    }

    #[test]
    fn flatten_shows_completed_when_filter_allows() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_completed_task(2, None, "b"),
        ]);
        let nodes = tree.flatten(|_| true, &[]);
        assert_eq!(task_ids(&nodes), vec![1, 2]);
    }

    #[test]
    fn flatten_active_before_someday_before_completed() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_someday_task(2, None, "b"),
            make_completed_task(3, None, "c"),
        ]);
        let nodes = tree.flatten(|_| true, &[]);
        assert_eq!(task_ids(&nodes), vec![1, 2, 3]);
    }

    #[test]
    fn flatten_ancestor_preservation() {
        // Root A (active) → Child B (completed) → Grandchild C (active)
        // Filter hides completed. C should pull B into view as a path node.
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_completed_task(2, Some(1), "a"),
            make_task(3, Some(2), "a"),
        ]);
        let nodes = tree.flatten(|t| t.completed_at.is_none(), &[]);
        assert_eq!(task_ids(&nodes), vec![1, 2, 3]);
        assert_eq!(nodes[0].depth, 0); // root
        assert_eq!(nodes[1].depth, 1); // preserved ancestor
        assert_eq!(nodes[2].depth, 2); // matched grandchild
    }

    #[test]
    fn flatten_root_must_pass_filter() {
        // Root (completed, fails filter) → Child (active, passes filter)
        // Root doesn't pass, so child is also hidden.
        let tree = TaskTree::build(vec![
            make_completed_task(1, None, "a"),
            make_task(2, Some(1), "a"),
        ]);
        let nodes = tree.flatten(|t| t.completed_at.is_none(), &[]);
        assert!(nodes.is_empty());
    }

    #[test]
    fn flatten_extra_visible_bypasses_filter() {
        let tree = TaskTree::build(vec![
            make_completed_task(1, None, "a"),
            make_task(2, None, "b"),
        ]);
        // Task 1 fails filter (completed) but is in extra_visible.
        // Display order: active (2) → completed (1).
        let nodes = tree.flatten(|t| t.completed_at.is_none(), &[1]);
        assert_eq!(task_ids(&nodes), vec![2, 1]);
    }

    #[test]
    fn flatten_extra_visible_includes_ancestors() {
        // Root A → Child B → Grandchild C (in extra_visible)
        // All fail filter. C's ancestors should be shown.
        let tree = TaskTree::build(vec![
            make_completed_task(1, None, "a"),
            make_completed_task(2, Some(1), "a"),
            make_completed_task(3, Some(2), "a"),
        ]);
        let nodes = tree.flatten(|_| false, &[3]);
        assert_eq!(task_ids(&nodes), vec![1, 2, 3]);
    }

    #[test]
    fn flatten_extra_visible_subtree_scanned() {
        // Root (in extra_visible, fails filter) → Child (passes filter)
        // Root visible via extra, child visible via filter.
        let tree = TaskTree::build(vec![
            make_completed_task(1, None, "a"),
            make_task(2, Some(1), "a"),
        ]);
        let nodes = tree.flatten(|t| t.completed_at.is_none(), &[1]);
        assert_eq!(task_ids(&nodes), vec![1, 2]);
    }

    // ── Flatten flat ───────────────────────────────────────────

    #[test]
    fn flatten_flat_preserves_order() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_task(2, None, "b"),
            make_task(3, None, "c"),
        ]);
        let nodes = tree.flatten_flat(&[3, 1]);
        assert_eq!(task_ids(&nodes), vec![3, 1]);
        assert_eq!(nodes[0].depth, 0);
        assert_eq!(nodes[1].depth, 0);
    }

    #[test]
    fn flatten_flat_skips_missing() {
        let tree = TaskTree::build(vec![make_task(1, None, "a")]);
        let nodes = tree.flatten_flat(&[1, 99]);
        assert_eq!(task_ids(&nodes), vec![1]);
    }

    // ── Child group ordering in flatten ────────────────────────

    #[test]
    fn flatten_children_ordered_active_someday_completed() {
        let tree = TaskTree::build(vec![
            make_task(1, None, "a"),
            make_completed_task(4, Some(1), "a"),
            make_someday_task(3, Some(1), "b"),
            make_task(2, Some(1), "c"),
        ]);
        let nodes = tree.flatten(|_| true, &[]);
        // Active (2), someday (3), completed (4)
        assert_eq!(task_ids(&nodes), vec![1, 2, 3, 4]);
    }
}
