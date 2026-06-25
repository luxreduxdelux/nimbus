use serde::{Deserialize, Serialize};

//================================================================

use crate::channel::*;

//================================================================

pub type AccountID = u64;
pub type AccountKey = [u8; 32];

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Account {
    pub key: AccountKey,
    pub name_nick: String,
    pub name_user: String,
    pub info: String,
    pub icon_main: Option<Vec<u8>>,
    pub icon_side: Option<Vec<u8>>,
    pub index: AccountID,
    pub channel: ChannelID,
    pub activity: Option<AccountActivity>,
    pub state: AccountState,
    pub write: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountState {
    #[default]
    Online,
    Away,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountActivity {
    App(String),
    Game(String),
    Video(String),
    Audio(String),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AccountConnect {
    pub key: AccountKey,
    pub name_nick: String,
    pub name_user: String,
    pub info: String,
    pub icon_main: Option<Vec<u8>>,
    pub icon_side: Option<Vec<u8>>,
}

pub enum NickError {
    Empty,
    Length,
}
pub enum UserError {
    Empty,
    Length,
    InvalidASCII,
}
pub enum InfoError {
    Length,
}
pub enum IconError {
    Length,
}

impl AccountConnect {
    pub const LIMIT_NICK_NAME: usize = 32;
    pub const LIMIT_USER_NAME: usize = 32;
    pub const LIMIT_USER_INFO: usize = 128;
    pub const LIMIT_USER_ICON: usize = 2_000_000;

    pub fn is_valid(&self) -> bool {
        Self::is_valid_nick(&self.name_nick).is_ok()
            && Self::is_valid_user(&self.name_user).is_ok()
            && Self::is_valid_info(&self.info).is_ok()
            && Self::is_valid_icon(&self.icon_main).is_ok()
    }

    pub fn is_valid_nick(name: &str) -> Result<(), NickError> {
        if name.is_empty() {
            return Err(NickError::Empty);
        }

        if name.len() > Self::LIMIT_NICK_NAME {
            return Err(NickError::Length);
        }

        Ok(())
    }

    pub fn is_valid_user(user: &str) -> Result<(), UserError> {
        if user.is_empty() {
            return Err(UserError::Empty);
        }

        if user.len() > Self::LIMIT_USER_NAME {
            return Err(UserError::Length);
        }

        if !user.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(UserError::InvalidASCII);
        }

        Ok(())
    }

    pub fn is_valid_info(info: &str) -> Result<(), InfoError> {
        if info.len() > Self::LIMIT_USER_INFO {
            return Err(InfoError::Length);
        }

        Ok(())
    }

    pub fn is_valid_icon(icon: &Option<Vec<u8>>) -> Result<(), IconError> {
        if let Some(icon) = icon {
            if icon.len() > Self::LIMIT_USER_ICON {
                return Err(IconError::Length);
            }
        }

        Ok(())
    }

    pub fn into_account(self, index: u64) -> Account {
        // TO-DO do truncation on self.name_* and such
        Account {
            key: self.key,
            name_nick: self.name_nick,
            name_user: self.name_user,
            info: self.info,
            icon_main: self.icon_main,
            icon_side: self.icon_side,
            index,
            channel: Default::default(),
            activity: Default::default(),
            state: Default::default(),
            write: Default::default(),
        }
    }
}
