use serde::{Deserialize, Serialize};

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::server::*;

//================================================================

pub type MessageID = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub channel: ChannelID,
    pub account: AccountID,
    pub star: bool,
    pub kind: MessageKind,
    pub reply: Option<AccountID>,
    pub react: Vec<(AccountID, char)>,
}

impl<'a> Message {
    pub fn account(&'a self, server: &'a Server) -> &'a Account {
        &server.account[&self.account]
    }

    pub fn new(
        channel: ChannelID,
        account: AccountID,
        kind: MessageKind,
        reply: Option<AccountID>,
    ) -> Self {
        Self {
            channel,
            account,
            star: Default::default(),
            kind,
            reply,
            react: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    Text(String),
    File(String, Vec<u8>),
    Sticker(u64),
}
