use serde::{Deserialize, Serialize};

//================================================================

use crate::storage::*;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
