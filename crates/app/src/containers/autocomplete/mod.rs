mod container;
pub(crate) mod view;

pub use container::AutocompleteInput;
pub use view::{find_trigger, get_suggestions, insert_completion, TriggerState};
