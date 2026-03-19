// Re-export FlatNode from stores (canonical definition).
pub use north_stores::FlatNode;

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
    CreateTop,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_siblings() {
        let flat = vec![
            FlatNode {
                task_id: 1,
                parent_id: None,
                depth: 0,
                is_completed: false,
                is_someday: false,
            },
            FlatNode {
                task_id: 3,
                parent_id: Some(1),
                depth: 1,
                is_completed: false,
                is_someday: false,
            },
            FlatNode {
                task_id: 4,
                parent_id: Some(1),
                depth: 1,
                is_completed: false,
                is_someday: false,
            },
            FlatNode {
                task_id: 2,
                parent_id: None,
                depth: 0,
                is_completed: false,
                is_someday: false,
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
                is_someday: false,
            },
            FlatNode {
                task_id: 3,
                parent_id: Some(1),
                depth: 1,
                is_completed: false,
                is_someday: false,
            },
        ];

        assert_eq!(first_child(&flat, 1), Some(3));
        assert_eq!(first_child(&flat, 3), None);
        assert_eq!(parent_of(&flat, 3), Some(1));
        assert_eq!(parent_of(&flat, 1), None);
    }
}
