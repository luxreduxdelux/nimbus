use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::message::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    // TO-DO make private
    pub count_account: u64,
    // TO-DO make private
    pub count_channel: u64,
    // TO-DO make private
    pub count_sticker: u64,
    pub account: BTreeMap<u64, Account>,
    pub channel: BTreeMap<u64, Channel>,
    pub sticker: BTreeMap<u64, Sticker>,
}

impl Server {
    const PATH_FILE: &str = "server.data";

    pub fn push_account(&mut self, account: Account) {
        self.account.insert(self.count_account, account);
        self.count_account += 1;
    }

    pub fn push_channel(&mut self, channel: Channel) {
        self.channel.insert(self.count_channel, channel);
        self.count_channel += 1;
    }

    pub fn set_channel_name(&mut self, channel: u64, name: &str) {}
    pub fn set_channel_info(&mut self, channel: u64, info: &str) {}

    pub fn push_message(&mut self, channel: u64, message: Message) {
        if let Some(channel) = self.channel.get_mut(&channel) {
            channel.message.insert(channel.count_message, message);
            channel.count_message += 1;
        }
    }

    pub fn push_sticker(&mut self, sticker: Sticker) {
        self.sticker.insert(self.count_sticker, sticker);
        self.count_sticker += 1;
    }

    pub fn set_account_state(&mut self, account: u64, state: AccountState) {
        if let Some(account) = self.account.get_mut(&account) {
            account.state = state;
        }
    }

    pub fn set_account_write(&mut self, account: u64, write: bool) {
        if let Some(account) = self.account.get_mut(&account) {
            account.write = write;
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        if let Ok(data) = std::fs::read(Self::PATH_FILE)
            && let Ok((user, _)) =
                bincode::serde::decode_from_slice(&data, bincode::config::standard())
        {
            user
        } else {
            let mut this = Self {
                count_account: Default::default(),
                count_channel: Default::default(),
                count_sticker: Default::default(),
                account: Default::default(),
                channel: Default::default(),
                sticker: Default::default(),
            };

            this.push_channel(Channel::default());

            this
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        //std::fs::write(
        //    Self::PATH_FILE,
        //    bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap(),
        //)
        //.unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sticker {
    pub data: Vec<u8>,
}
