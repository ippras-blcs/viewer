use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct State {
    pub(crate) open_settings_window: bool,
    pub(crate) reset_table_state: bool,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            open_settings_window: false,
            reset_table_state: false,
        }
    }
}
