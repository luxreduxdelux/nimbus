use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

//================================================================

use crate::account::*;
use crate::category::*;
use crate::channel::*;
use crate::configuration::*;
use crate::message::*;
use crate::sticker::*;

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub version: u64,
    pub count_account: u64,
    pub count_channel: u64,
    pub count_sticker: u64,
    pub configuration: Configuration,
    pub account_key: BTreeMap<AccountKey, AccountID>,
    pub account: BTreeMap<AccountID, Account>,
    pub channel: BTreeMap<ChannelID, Channel>,
    pub sticker: BTreeMap<StickerID, Sticker>,
    pub private: BTreeMap<(AccountID, AccountID), Channel>,
    pub category: Vec<Category>,
    pub name: String,
    pub info: String,
    pub icon: Option<Vec<u8>>,
}

impl Server {
    const DEFAULT_NAME: &str = "Nimbus Server";
    const DEFAULT_INFO: &str = "A default Nimbus server, for the people, by the people.\nhttps://github.com/luxreduxdelux/nimbus";

    //================================================================

    pub fn push_account(&mut self, account: Account) {
        self.account.insert(self.count_account, account);
        self.count_account += 1;
    }

    /*
    pub fn delete_account(&mut self, account: ChannelID) {
        // TO-DO should search every single message by this account and delete it
    }
    */

    pub fn set_account_channel(&mut self, account: AccountID, channel: ChannelID) {
        if let Some(account) = self.account.get_mut(&account) {
            account.channel = channel;
        }
    }

    pub fn set_account_presence(&mut self, account: AccountID, presence: AccountPresence) {
        if let Some(account) = self.account.get_mut(&account) {
            account.presence = presence;
        }
    }

    pub fn set_account_state(&mut self, account: AccountID, state: Option<String>) {
        if let Some(account) = self.account.get_mut(&account) {
            account.state = state;
        }
    }

    pub fn set_account_write(&mut self, account: AccountID, write: bool) {
        if let Some(account) = self.account.get_mut(&account) {
            account.write = write;
        }
    }

    //================================================================

    pub fn push_channel(&mut self, channel: Channel) {
        self.channel.insert(self.count_channel, channel);
        self.count_channel += 1;
    }

    pub fn delete_channel(&mut self, channel: ChannelID) {
        self.channel.remove(&channel);
    }

    pub fn set_channel_name(&mut self, channel: ChannelID, name: &str) {}
    pub fn set_channel_info(&mut self, channel: ChannelID, info: &str) {}

    //================================================================

    pub fn push_category(&mut self, category: Category) {
        self.category.push(category);
    }

    pub fn delete_category(&mut self, index: usize) {
        self.category.remove(index);
    }

    //================================================================

    pub fn push_message(&mut self, channel: ChannelID, message: Message) {
        if let Some(channel) = self.channel.get_mut(&channel) {
            channel.message.insert(channel.count_message, message);
            channel.count_message += 1;
        }
    }

    pub fn delete_message(&mut self, channel: ChannelID, message: MessageID) {
        if let Some(channel) = self.channel.get_mut(&channel) {
            channel.message.remove(&message);
        }
    }

    //================================================================

    pub fn poll_vote(
        &mut self,
        account: AccountID,
        channel: ChannelID,
        message: MessageID,
        choice: usize,
    ) {
        if let Some(channel) = self.channel.get_mut(&channel)
            && let Some(message) = channel.message.get_mut(&message)
            && let MessageKind::Poll(poll) = &mut message.kind
            && let Some(choice) = poll.choice.get_mut(choice)
        {
            if choice.vote.contains(&account) {
                choice.vote.remove(&account);
            } else {
                choice.vote.insert(account);
            }
        }
    }

    //================================================================

    pub fn push_sticker(&mut self, sticker: Sticker) {
        self.sticker.insert(self.count_sticker, sticker);
        self.count_sticker += 1;
    }

    //================================================================

    pub fn load(path: &str) -> Self {
        // TO-DO handle scenario where we could not deserialize but file does exist
        if let Ok(data) = std::fs::read(path)
            && let Ok((user, _)) =
                bincode::serde::decode_from_slice(&data, bincode::config::standard())
        {
            user
        } else {
            Self::default()
        }
    }

    pub fn save(&self, path: &str) {
        std::fs::write(
            path,
            bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap(),
        )
        .unwrap();
    }
}

impl Default for Server {
    fn default() -> Self {
        let mut this = Self {
            version: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            count_account: Default::default(),
            count_channel: Default::default(),
            count_sticker: Default::default(),
            configuration: Default::default(),
            account_key: Default::default(),
            account: Default::default(),
            channel: Default::default(),
            sticker: Default::default(),
            private: Default::default(),
            category: Default::default(),
            name: Self::DEFAULT_NAME.to_string(),
            info: Self::DEFAULT_INFO.to_string(),
            icon: None,
        };

        this.push_category(Category::new("General".to_string(), vec![0, 1, 2]));

        this.push_channel(Channel::new("foo"));
        this.push_channel(Channel::new("bar"));
        this.push_channel(Channel::new("baz"));
        this.push_channel(Channel::new("qux"));

        this
    }
}
