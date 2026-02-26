use north_stores::TaskModel;

/// Compute whether a task is actionable based on its parent's sequential_limit.
/// Root tasks are always actionable. Subtasks are actionable only if they are
/// within the first N incomplete siblings (sorted by sort_key) where N is the
/// parent's sequential_limit. A limit of 0 means unlimited.
pub fn is_actionable(task: &TaskModel, all_tasks: &[TaskModel]) -> bool {
    if task.someday {
        return false;
    }
    let Some(parent_id) = task.parent_id else {
        return true;
    };
    let limit = all_tasks
        .iter()
        .find(|t| t.id == parent_id)
        .map(|p| p.sequential_limit)
        .unwrap_or(1);
    if limit == 0 {
        return true;
    }
    let siblings_before = all_tasks
        .iter()
        .filter(|t| {
            t.parent_id == Some(parent_id) && t.completed_at.is_none() && t.sort_key < task.sort_key
        })
        .count();
    (siblings_before as i16) < limit
}
