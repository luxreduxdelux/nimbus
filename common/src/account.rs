use serde::{Deserialize, Serialize};

//================================================================

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Account {
    //pub key: u128,
    pub name: String,
    pub icon: Option<Vec<u8>>,
    pub state: AccountState,
    pub write: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum AccountState {
    #[default]
    Online,
    Away,
    DoNotDisturb,
    Offline,
}
