use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

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
    pub react: HashMap<char, (AccountID, u64)>,
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
    Poll(Poll),
    Sticker(u64),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
// anonymous poll
// single answer
// add new option
// allow re-vote
// set correct answer
// limit duration + hide result until end
pub struct Poll {
    pub name: String,
    pub choice: Vec<PollChoice>,
    pub hidden: bool,
    pub single: bool,
    pub attach: bool,
    pub revoke: bool,
    pub correct: Option<usize>,
}

pub enum PollError {
    NameLengthZero,
    NameLength,
    ChoiceLengthZero,
    ChoiceLength,
    ChoiceNameEmpty,
    ChoiceNameLength,
    InvalidCorrectIndex,
}

impl Poll {
    const LIMIT_TEXT: usize = 24;
    const LIMIT_CHOICE: usize = 10;

    pub fn is_valid(&self) -> Result<(), PollError> {
        if self.name.is_empty() {
            return Err(PollError::NameLengthZero);
        }

        if self.name.len() > Self::LIMIT_TEXT {
            return Err(PollError::NameLength);
        }

        if self.choice.is_empty() {
            return Err(PollError::ChoiceLengthZero);
        }

        if self.choice.len() > Self::LIMIT_CHOICE {
            return Err(PollError::ChoiceLength);
        }

        // TO-DO rest

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PollChoice {
    pub name: String,
    // TO-DO client should not be able to manipulate this, only read it
    pub vote: HashSet<AccountID>,
}
