use serde::{Deserialize, Serialize};

//================================================================

pub type RoleID = u64;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub color: Color,
    pub display: bool,
    pub mention: bool,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
