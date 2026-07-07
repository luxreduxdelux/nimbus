use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::configuration::*;
use crate::message::*;
use crate::storage::*;

//================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Server {
    pub configuration: Configuration,
    pub account: BTreeMap<AccountID, Account>,
    pub channel: BTreeMap<ChannelID, Channel>,
    pub message: BTreeMap<MessageID, Message>,
    pub view: HashMap<ChannelID, Vec<MessageID>>,
    pub name: String,
    pub info: String,
    pub icon: Option<Vec<u8>>,
}

impl Server {
    const DEFAULT_NAME: &str = "Nimbus Server";
    const DEFAULT_INFO: &str = "A default Nimbus server, for the people, by the people.\nhttps://github.com/luxreduxdelux/nimbus";

    //================================================================

    pub fn push_account(&mut self, account: Account) {
        self.account.insert(account.index, account);
    }

    /*
    pub fn delete_account(&mut self, account: AccountID) {
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
        self.channel.insert(channel.index, channel);
    }

    pub fn delete_channel(&mut self, channel: ChannelID) {
        self.channel.remove(&channel);
    }

    pub fn set_channel_name(&mut self, channel: ChannelID, name: &str) {}
    pub fn set_channel_info(&mut self, channel: ChannelID, info: &str) {}

    //================================================================

    pub fn push_message(&mut self, message: Message) {
        self.message.insert(message.index, message);
    }

    pub fn delete_message(&mut self, message: MessageID) {
        self.message.remove(&message);
    }

    pub fn poll_vote(&mut self, account: AccountID, message: MessageID, choice: usize) {
        if let Some(message) = self.message.get_mut(&message)
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

    pub fn from_storage(storage: &Storage) -> anyhow::Result<Self> {
        Ok(Self {
            configuration: Default::default(),
            account: storage.get_all_account()?,
            channel: storage.get_all_channel()?,
            message: storage.get_all_message()?,
            view: Default::default(),
            name: Self::DEFAULT_NAME.to_string(),
            info: Self::DEFAULT_INFO.to_string(),
            icon: Default::default(),
        })
    }
}
