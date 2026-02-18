use leptos::prelude::*;

use crate::AppStore;
use crate::ModalStore;
use crate::TaskDetailModalStore;

pub fn use_app_store() -> AppStore {
    expect_context::<AppStore>()
}

pub fn use_task_detail_modal_store() -> TaskDetailModalStore {
    expect_context::<TaskDetailModalStore>()
}

pub fn use_modal_store() -> ModalStore {
    expect_context::<ModalStore>()
}
