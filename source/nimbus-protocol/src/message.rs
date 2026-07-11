use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

//================================================================

use crate::account::*;
use crate::cache::*;
use crate::channel::*;
use crate::server::*;
use crate::token::*;

//================================================================

pub type MessageID = (ChannelID, u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub index: MessageID,
    pub account: Option<AccountID>,
    pub star: bool,
    pub kind: MessageKind,
    pub reply: Option<MessageID>,
    pub react: HashMap<char, (AccountID, u64)>,
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
        if let MessageKind::Text(text) = &self.kind {
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
        kind: MessageKind,
        reply: Option<MessageID>,
    ) -> Self {
        Self {
            index,
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
    System(MessageSystem),
    Text(String),
    File(String, Vec<u8>),
    Poll(Poll),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSystem {
    Enter(AccountID),
    Leave(AccountID),
    Star(MessageID),
}

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

impl MessageKind {
    #[rustfmt::skip]
    pub fn is_valid(&self, server: &Server) -> Result<(), MessageError> {
        match self {
            MessageKind::Text(text)       => Self::is_valid_text(server, text),
            MessageKind::File(name, data) => Self::is_valid_file(server, name, data),
            MessageKind::Poll(poll)       => Self::is_valid_poll(server, poll),
            _ => Ok(())
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

    pub fn is_valid_file(server: &Server, name: &str, data: &[u8]) -> Result<(), MessageError> {
        Self::is_valid_text(server, name)?;

        if data.is_empty() {
            return Err(MessageError::FileEmpty);
        }

        if data.len() > server.configuration.limit_file_size {
            return Err(MessageError::FileLength);
        }

        Ok(())
    }

    pub fn is_valid_poll(server: &Server, poll: &Poll) -> Result<(), MessageError> {
        Self::is_valid_text(server, &poll.name)?;

        if poll.choice.is_empty() {
            return Err(MessageError::PollChoiceEmpty);
        }

        if poll.choice.len() > server.configuration.limit_poll_size {
            return Err(MessageError::PollChoiceLength);
        }

        for choice in &poll.choice {
            Self::is_valid_text(server, &choice.name)?;
        }

        if let Some(correct) = poll.correct
            && correct > poll.choice.len()
        {
            return Err(MessageError::PollInvalidCorrectIndex);
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
