use north_dto::Task;

#[derive(Clone, Debug, PartialEq)]
pub struct FlatNode {
    pub task_id: i64,
    pub parent_id: Option<i64>,
    pub depth: u8,
    pub is_completed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Placement {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InlineMode {
    None,
    Edit {
        task_id: i64,
    },
    Create {
        anchor_task_id: i64,
        placement: Placement,
        parent_id: Option<i64>,
        depth: u8,
    },
}

/// Build a flat traversal list from a set of root task IDs and all tasks.
/// DFS pre-order: per parent group, active tasks sorted by sort_key come first,
/// then completed tasks (when show_completed is on).
pub fn flatten_tree(root_ids: &[i64], all_tasks: &[Task], show_completed: bool) -> Vec<FlatNode> {
    let mut nodes = Vec::new();

    let mut roots: Vec<&Task> = root_ids
        .iter()
        .filter_map(|id| all_tasks.iter().find(|t| t.id == *id))
        .collect();

    let (mut active, mut completed): (Vec<&Task>, Vec<&Task>) =
        roots.drain(..).partition(|t| t.completed_at.is_none());

    active.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
    for task in &active {
        flatten_subtree(task, all_tasks, 0, show_completed, &mut nodes);
    }

    if show_completed {
        completed.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        for task in &completed {
            flatten_subtree(task, all_tasks, 0, show_completed, &mut nodes);
        }
    }

    nodes
}

fn flatten_subtree(
    task: &Task,
    all_tasks: &[Task],
    depth: u8,
    show_completed: bool,
    nodes: &mut Vec<FlatNode>,
) {
    nodes.push(FlatNode {
        task_id: task.id,
        parent_id: task.parent_id,
        depth,
        is_completed: task.completed_at.is_some(),
    });

    let children: Vec<&Task> = all_tasks
        .iter()
        .filter(|t| t.parent_id == Some(task.id))
        .collect();

    let (mut active, mut completed): (Vec<&Task>, Vec<&Task>) =
        children.into_iter().partition(|t| t.completed_at.is_none());

    active.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
    for child in &active {
        flatten_subtree(child, all_tasks, depth + 1, show_completed, nodes);
    }

    if show_completed {
        completed.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        for child in &completed {
            flatten_subtree(child, all_tasks, depth + 1, show_completed, nodes);
        }
    }
}

// ── Navigation helpers ─────────────────────────────────────────

/// Previous sibling (same parent_id) in flat order.
pub fn prev_sibling(flat: &[FlatNode], task_id: i64) -> Option<i64> {
    let idx = flat.iter().position(|n| n.task_id == task_id)?;
    let parent_id = flat[idx].parent_id;
    (0..idx)
        .rev()
        .find(|&i| flat[i].parent_id == parent_id)
        .map(|i| flat[i].task_id)
}

/// Next sibling (same parent_id) in flat order.
pub fn next_sibling(flat: &[FlatNode], task_id: i64) -> Option<i64> {
    let idx = flat.iter().position(|n| n.task_id == task_id)?;
    let parent_id = flat[idx].parent_id;
    ((idx + 1)..flat.len())
        .find(|&i| flat[i].parent_id == parent_id)
        .map(|i| flat[i].task_id)
}

/// First child of the given task in flat order.
pub fn first_child(flat: &[FlatNode], task_id: i64) -> Option<i64> {
    flat.iter()
        .find(|n| n.parent_id == Some(task_id))
        .map(|n| n.task_id)
}

/// Parent task ID of the given node.
pub fn parent_of(flat: &[FlatNode], task_id: i64) -> Option<i64> {
    flat.iter()
        .find(|n| n.task_id == task_id)
        .and_then(|n| n.parent_id)
}

// ── Sort key computation ───────────────────────────────────────

/// Compute a sort_key for a new task being inserted relative to an anchor.
/// `parent_id` is the intended parent for the new task (may differ from
/// anchor's parent when indented/outdented).
pub fn compute_sort_key(
    flat: &[FlatNode],
    all_tasks: &[Task],
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
                let above_key = task_sort_key(all_tasks, anchor_task_id);
                let below_key = siblings
                    .get(anchor_pos + 1)
                    .and_then(|&id| task_sort_key(all_tasks, id));
                north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref())
            }
            Placement::Before => {
                let above_key = if anchor_pos > 0 {
                    siblings
                        .get(anchor_pos - 1)
                        .and_then(|&id| task_sort_key(all_tasks, id))
                } else {
                    None
                };
                let below_key = task_sort_key(all_tasks, anchor_task_id);
                north_dto::sort_key_between(above_key.as_deref(), below_key.as_deref())
            }
        }
    } else {
        // Anchor is not a sibling at this depth (e.g. after indent).
        // Place at end of existing siblings.
        let last_key = siblings.last().and_then(|&id| task_sort_key(all_tasks, id));
        north_dto::sort_key_after(last_key.as_deref())
    }
}

pub fn task_sort_key(all_tasks: &[Task], task_id: i64) -> Option<String> {
    all_tasks
        .iter()
        .find(|t| t.id == task_id)
        .map(|t| t.sort_key.clone())
}

// ── Depth helpers ──────────────────────────────────────────────

/// Maximum depth allowed for a new task being created at the given anchor.
pub fn max_create_depth(flat: &[FlatNode], anchor_task_id: i64, placement: Placement) -> u8 {
    let Some(anchor_idx) = flat.iter().position(|n| n.task_id == anchor_task_id) else {
        return 0;
    };
    match placement {
        Placement::After => flat[anchor_idx].depth + 1,
        Placement::Before => {
            if anchor_idx > 0 {
                flat[anchor_idx - 1].depth + 1
            } else {
                0
            }
        }
    }
}

/// Walk backwards from the anchor to find the parent task for a given depth.
pub fn find_parent_for_depth(
    flat: &[FlatNode],
    anchor_task_id: i64,
    target_depth: u8,
) -> Option<i64> {
    if target_depth == 0 {
        return None;
    }
    let anchor_idx = flat.iter().position(|n| n.task_id == anchor_task_id)?;
    for i in (0..=anchor_idx).rev() {
        if flat[i].depth == target_depth - 1 {
            return Some(flat[i].task_id);
        }
        if flat[i].depth < target_depth.saturating_sub(1) {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use north_dto::Task;

    fn make_task(id: i64, parent_id: Option<i64>, sort_key: &str) -> Task {
        Task {
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
            project_title: None,
            tags: vec![],
            subtask_count: 0,
            completed_subtask_count: 0,
            actionable: true,
        }
    }

    fn make_completed_task(id: i64, parent_id: Option<i64>, sort_key: &str) -> Task {
        let mut t = make_task(id, parent_id, sort_key);
        t.completed_at = Some(Utc::now());
        t
    }

    #[test]
    fn flatten_tree_basic() {
        let tasks = vec![
            make_task(1, None, "a"),
            make_task(2, None, "b"),
            make_task(3, Some(1), "a"),
            make_task(4, Some(1), "b"),
        ];
        let roots = vec![1, 2];
        let flat = flatten_tree(&roots, &tasks, false);

        assert_eq!(flat.len(), 4);
        assert_eq!(flat[0].task_id, 1);
        assert_eq!(flat[0].depth, 0);
        assert_eq!(flat[1].task_id, 3);
        assert_eq!(flat[1].depth, 1);
        assert_eq!(flat[2].task_id, 4);
        assert_eq!(flat[2].depth, 1);
        assert_eq!(flat[3].task_id, 2);
        assert_eq!(flat[3].depth, 0);
    }

    #[test]
    fn flatten_tree_hides_completed() {
        let tasks = vec![make_task(1, None, "a"), make_completed_task(2, None, "b")];
        let roots = vec![1, 2];

        let flat = flatten_tree(&roots, &tasks, false);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].task_id, 1);

        let flat = flatten_tree(&roots, &tasks, true);
        assert_eq!(flat.len(), 2);
    }

    #[test]
    fn navigation_siblings() {
        let flat = vec![
            FlatNode {
                task_id: 1,
                parent_id: None,
                depth: 0,
                is_completed: false,
            },
            FlatNode {
                task_id: 3,
                parent_id: Some(1),
                depth: 1,
                is_completed: false,
            },
            FlatNode {
                task_id: 4,
                parent_id: Some(1),
                depth: 1,
                is_completed: false,
            },
            FlatNode {
                task_id: 2,
                parent_id: None,
                depth: 0,
                is_completed: false,
            },
        ];

        assert_eq!(next_sibling(&flat, 1), Some(2));
        assert_eq!(prev_sibling(&flat, 2), Some(1));
        assert_eq!(next_sibling(&flat, 2), None);
        assert_eq!(prev_sibling(&flat, 1), None);

        assert_eq!(next_sibling(&flat, 3), Some(4));
        assert_eq!(prev_sibling(&flat, 4), Some(3));
    }

    #[test]
    fn navigation_parent_child() {
        let flat = vec![
            FlatNode {
                task_id: 1,
                parent_id: None,
                depth: 0,
                is_completed: false,
            },
            FlatNode {
                task_id: 3,
                parent_id: Some(1),
                depth: 1,
                is_completed: false,
            },
        ];

        assert_eq!(first_child(&flat, 1), Some(3));
        assert_eq!(first_child(&flat, 3), None);
        assert_eq!(parent_of(&flat, 3), Some(1));
        assert_eq!(parent_of(&flat, 1), None);
    }
}
