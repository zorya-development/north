use leptos::prelude::*;

use crate::AppStore;

pub fn use_app_store() -> AppStore {
    expect_context::<AppStore>()
}
