use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::server::*;
use crate::sticker::*;

//================================================================

pub type MessageID = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub channel: ChannelID,
    pub account: AccountID,
    pub star: bool,
    pub kind: MessageKind,
    pub reply: Option<MessageID>,
    pub react: HashMap<char, (AccountID, u64)>,
}

impl<'a> Message {
    pub fn account(&'a self, server: &'a Server) -> Option<&'a Account> {
        server.account.get(&self.account)
    }

    pub fn new(
        channel: ChannelID,
        account: AccountID,
        kind: MessageKind,
        reply: Option<MessageID>,
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageError {
    TextEmpty,
    TextLength,
    FileEmpty,
    FileLength,
    PollNameEmpty,
    PollNameLength,
    PollChoiceEmpty,
    PollChoiceLength,
    PollChoiceNameEmpty,
    PollChoiceNameLength,
    PollInvalidCorrectIndex,
    StickerInvalidIndex,
}

impl MessageKind {
    const TEXT_LIMIT_TEXT: usize = 256;
    const FILE_LIMIT_DATA: usize = 1_000_000 * 10;
    const POLL_LIMIT_TEXT: usize = 64;
    const POLL_LIMIT_CHOICE: usize = 16;

    #[rustfmt::skip]
    pub fn is_valid(&self, server: &Server) -> Result<(), MessageError> {
        match self {
            MessageKind::Text(text)       => Self::is_valid_text(text),
            MessageKind::File(name, data) => Self::is_valid_file(name, data),
            MessageKind::Poll(poll)       => Self::is_valid_poll(poll),
            MessageKind::Sticker(index)   => Self::is_valid_sticker(index, server),
        }
    }

    pub fn is_valid_text(text: &str) -> Result<(), MessageError> {
        if text.is_empty() {
            return Err(MessageError::TextEmpty);
        }

        if text.len() > Self::TEXT_LIMIT_TEXT {
            return Err(MessageError::TextLength);
        }

        Ok(())
    }

    pub fn is_valid_file(name: &str, data: &[u8]) -> Result<(), MessageError> {
        Self::is_valid_text(name)?;

        if data.is_empty() {
            return Err(MessageError::FileEmpty);
        }

        if data.len() > Self::FILE_LIMIT_DATA {
            return Err(MessageError::FileLength);
        }

        Ok(())
    }

    pub fn is_valid_poll(poll: &Poll) -> Result<(), MessageError> {
        if poll.name.is_empty() {
            return Err(MessageError::PollNameEmpty);
        }

        if poll.name.len() > Self::POLL_LIMIT_TEXT {
            return Err(MessageError::PollNameLength);
        }

        if poll.choice.is_empty() {
            return Err(MessageError::PollChoiceEmpty);
        }

        if poll.choice.len() > Self::POLL_LIMIT_CHOICE {
            return Err(MessageError::PollChoiceLength);
        }

        // TO-DO rest

        Ok(())
    }

    pub fn is_valid_sticker(index: &StickerID, server: &Server) -> Result<(), MessageError> {
        if !server.sticker.contains_key(index) {
            return Err(MessageError::StickerInvalidIndex);
        }

        Ok(())
    }
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PollChoice {
    pub name: String,
    // TO-DO client should not be able to manipulate this, only read it
    pub vote: HashSet<AccountID>,
}
