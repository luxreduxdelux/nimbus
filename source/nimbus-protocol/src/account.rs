use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

//================================================================

use crate::channel::*;
use crate::command::Challenge;

//================================================================

pub type AccountID = u64;
pub type AccountKey = [u8; 32];

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Account {
    pub index: AccountID,
    pub key: AccountKey,
    pub name_nick: String,
    pub name_user: String,
    pub info: String,
    pub icon: Option<Vec<u8>>,
    // TO-DO make option? to signify we are not even in the server
    pub channel: ChannelID,
    pub activity: Option<AccountActivity>,
    pub presence: AccountPresence,
    pub state: Option<String>,
    pub write: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountPresence {
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
    pub icon: Option<Vec<u8>>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountError {
    NickEmpty,
    NickLength,
    NameEmpty,
    NameLength,
    NameInvalid,
    InfoLength,
    IconLength,
}

impl AccountConnect {
    pub const LIMIT_NICK_NAME: usize = 64;
    pub const LIMIT_USER_NAME: usize = 64;
    pub const LIMIT_USER_INFO: usize = 256;
    pub const LIMIT_USER_ICON: usize = 1_000_000 * 2;

    pub fn is_valid(&self) -> Result<(), AccountError> {
        Self::is_valid_nick(&self.name_nick)?;
        Self::is_valid_name(&self.name_user)?;
        Self::is_valid_info(&self.info)?;
        Self::is_valid_icon(&self.icon)?;

        Ok(())
    }

    pub fn is_valid_nick(name: &str) -> Result<(), AccountError> {
        if name.is_empty() {
            return Err(AccountError::NickEmpty);
        }

        if name.len() > Self::LIMIT_NICK_NAME {
            return Err(AccountError::NickLength);
        }

        Ok(())
    }

    pub fn is_valid_name(user: &str) -> Result<(), AccountError> {
        if user.is_empty() {
            return Err(AccountError::NameEmpty);
        }

        if user.len() > Self::LIMIT_USER_NAME {
            return Err(AccountError::NameLength);
        }

        if !user.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(AccountError::NameInvalid);
        }

        Ok(())
    }

    pub fn is_valid_info(info: &str) -> Result<(), AccountError> {
        if info.len() > Self::LIMIT_USER_INFO {
            return Err(AccountError::InfoLength);
        }

        Ok(())
    }

    pub fn is_valid_icon(icon: &Option<Vec<u8>>) -> Result<(), AccountError> {
        if let Some(icon) = icon
            && icon.len() > Self::LIMIT_USER_ICON
        {
            return Err(AccountError::IconLength);
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
            icon: self.icon,
            index,
            channel: Default::default(),
            activity: Default::default(),
            presence: Default::default(),
            state: Default::default(),
            write: Default::default(),
        }
    }

    pub fn create_nonce() -> Challenge {
        let mut challenge = [0; 32];
        OsRng.fill_bytes(&mut challenge);
        challenge.to_vec()
    }

    pub fn verify_nonce(
        key: AccountKey,
        challenge: Challenge,
        signature: crate::command::Signature,
    ) -> bool {
        let v_key = VerifyingKey::from_bytes(&key).unwrap();
        let v_sig = Signature::from_bytes(signature.as_slice().try_into().unwrap());

        v_key.verify_strict(&challenge, &v_sig).is_ok()
    }
}
