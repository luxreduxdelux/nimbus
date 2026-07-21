use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, VerifyingKey};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};

//================================================================

use crate::channel::*;
use crate::command::*;
use crate::role::*;
use crate::utility::*;

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
    pub role: Vec<RoleID>,
    // TO-DO make option? to signify we are not even in the server
    pub channel: ChannelID,
    pub activity: Option<AccountActivity>,
    pub presence: AccountPresence,
    pub state: Option<String>,
    pub write: bool,
}

impl Account {
    pub fn halo(&self) -> String {
        STANDARD.encode(self.key)
    }

    pub fn halo_short(&self) -> String {
        let halo = STANDARD.encode(&self.key[0..6]);

        format!("{}-{}", &halo[0..4], &halo[4..8])
    }

    pub fn halo_color(&self) -> Color {
        const HSV_S: f32 = 0.7;
        const HSV_V: f32 = 0.8;
        let value = u32::from_le_bytes([self.key[0], self.key[1], self.key[2], self.key[3]]);
        let h = (value as f32 / u32::MAX as f32) * 360.0;

        Self::convert_hsv(h, HSV_S, HSV_V)
    }

    fn convert_hsv(h: f32, s: f32, v: f32) -> Color {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = if (0.0..1.0).contains(&h_prime) {
            (c, x, 0.0)
        } else if (1.0..2.0).contains(&h_prime) {
            (x, c, 0.0)
        } else if (2.0..3.0).contains(&h_prime) {
            (0.0, c, x)
        } else if (3.0..4.0).contains(&h_prime) {
            (0.0, x, c)
        } else if (4.0..5.0).contains(&h_prime) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Color {
            r: ((r_prime + m) * 255.0) as u8,
            g: ((g_prime + m) * 255.0) as u8,
            b: ((b_prime + m) * 255.0) as u8,
        }
    }
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
    pub invite: Option<String>,
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
            index,
            key: self.key,
            name_nick: self.name_nick,
            name_user: self.name_user,
            info: self.info,
            icon: self.icon,
            role: Default::default(),
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
