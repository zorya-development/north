use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DropZone {
    Above,
    Below,
    Nest,
}

#[derive(Clone, Copy)]
pub struct DragDropContext {
    pub dragging_task_id: RwSignal<Option<i64>>,
    pub drop_target: RwSignal<Option<(i64, DropZone)>>,
}

impl Default for DragDropContext {
    fn default() -> Self {
        Self::new()
    }
}

impl DragDropContext {
    pub fn new() -> Self {
        Self {
            dragging_task_id: RwSignal::new(None),
            drop_target: RwSignal::new(None),
        }
    }
}
