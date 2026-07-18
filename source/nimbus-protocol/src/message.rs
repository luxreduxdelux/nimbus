use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

//================================================================

use crate::account::*;
use crate::cache::*;
use crate::channel::*;
use crate::file::*;
use crate::server::*;
use crate::stamp::*;
use crate::storage::*;
use crate::token::*;

//================================================================

pub type MessageID = (ChannelID, u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub index: MessageID,
    pub account: Option<AccountID>,
    pub date: DateTime<Utc>,
    pub star: bool,
    pub reply: Option<MessageID>,
    pub react: HashMap<char, (AccountID, u64)>,
    pub value: MessageValue,
}

impl<'a> Message {
    pub fn account(&'a self, cache: &'a mut Cache) -> Option<&'a Account> {
        if let Some(account) = self.account
            && let Some(account) = cache.get_account(account)
        {
            return Some(account);
        }

        None
    }

    pub fn is_mention(&self, account: &Account) -> bool {
        if let MessageValue::Text(text) = &self.value {
            let (list, _, _) = Token::parse(text);

            for token in list {
                if let Token::Account(mention) = token
                    && mention[1..mention.len()] == account.name_user
                {
                    return true;
                }
            }
        }

        false
    }

    pub fn new(
        index: MessageID,
        account: Option<AccountID>,
        value: MessageValue,
        reply: Option<MessageID>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            index,
            account,
            star: Default::default(),
            date: Utc::now(),
            reply,
            react: Default::default(),
            value,
        })
    }
}

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageValue {
    System(MessageSystem),
    Text(String),
    File(FileMeta),
    Poll(Poll),
    Stamp(StampID),
}

impl MessageValue {
    pub fn from_request(
        message: MessageValueRequest,
        storage: &mut Storage,
    ) -> anyhow::Result<Self> {
        Ok(match message {
            MessageValueRequest::Text(text) => Self::Text(text),
            MessageValueRequest::File(file) => MessageValue::File(file.insert(storage)?),
            MessageValueRequest::Poll(poll) => Self::Poll(Poll {
                vote: Default::default(),
                value: poll,
            }),
            MessageValueRequest::Stamp(stamp) => Self::Stamp(stamp),
        })
    }
}

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageValueRequest {
    Text(String),
    File(FileValue),
    Poll(PollValue),
    Stamp(StampID),
}

impl MessageValueRequest {
    #[rustfmt::skip]
    pub fn is_valid(&self, server: &Server) -> Result<(), MessageError> {
        match self {
            Self::Text(text)   => Self::is_valid_text(server, text),
            Self::File(file)   => Self::is_valid_file(server, file),
            Self::Poll(poll)   => Self::is_valid_poll(server, poll),
            Self::Stamp(stamp) => Self::is_valid_stamp(server, stamp),
        }
    }

    pub fn is_valid_text(server: &Server, text: &str) -> Result<(), MessageError> {
        if text.is_empty() {
            return Err(MessageError::TextEmpty);
        }

        if text.len() > server.configuration.limit_text_size {
            return Err(MessageError::TextLength);
        }

        Ok(())
    }

    pub fn is_valid_file(server: &Server, file: &FileValue) -> Result<(), MessageError> {
        Self::is_valid_text(server, &file.name)?;

        if file.data.is_empty() {
            return Err(MessageError::FileEmpty);
        }

        if file.data.len() > server.configuration.limit_file_size {
            return Err(MessageError::FileLength);
        }

        Ok(())
    }

    pub fn is_valid_poll(server: &Server, poll: &PollValue) -> Result<(), MessageError> {
        Self::is_valid_text(server, &poll.name)?;

        if poll.choice.is_empty() {
            return Err(MessageError::PollChoiceEmpty);
        }

        if poll.choice.len() > server.configuration.limit_poll_size {
            return Err(MessageError::PollChoiceLength);
        }

        for choice in &poll.choice {
            Self::is_valid_text(server, choice)?;
        }

        if let Some(correct) = poll.correct
            && correct > poll.choice.len()
        {
            return Err(MessageError::PollInvalidCorrectIndex);
        }

        Ok(())
    }

    pub fn is_valid_stamp(server: &Server, stamp: &StampID) -> Result<(), MessageError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSystem {
    Enter(AccountID),
    Leave(AccountID),
    Star(MessageID),
}

//================================================================

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageError {
    TextEmpty,
    TextLength,
    FileEmpty,
    FileLength,
    PollChoiceEmpty,
    PollChoiceLength,
    PollInvalidCorrectIndex,
}

//================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Poll {
    pub vote: HashMap<usize, HashSet<AccountID>>,
    pub value: PollValue,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PollValue {
    pub name: String,
    pub choice: Vec<String>,
    pub hidden: bool,
    pub single: bool,
    pub attach: bool,
    pub revoke: bool,
    pub correct: Option<usize>,
}
