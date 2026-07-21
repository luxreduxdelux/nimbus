use serde::{Deserialize, Serialize};

//================================================================

use crate::utility::*;

//================================================================

pub type RoleID = u64;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Role {
    pub index: RoleID,
    pub value: RoleValue,
}

impl Role {
    pub fn new(index: RoleID, value: RoleValue) -> Self {
        Self { index, value }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleValue {
    pub name: String,
    pub color: Color,
    pub channel_view: bool,
    pub channel_edit: bool,
}
