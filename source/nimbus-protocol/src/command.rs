use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::configuration::*;
use crate::emote::*;
use crate::file::*;
use crate::invite::*;
use crate::message::*;
use crate::role::*;
use crate::server::*;
use crate::stamp::*;
use crate::utility::*;

//================================================================

pub type Signature = Vec<u8>;

/// A command the client can send to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandClient {
    Enter(AccountConnect),
    Leave,
    Nonce(Signature),

    Configuration(Configuration),

    Channel(ChannelValue),
    ChannelModify(ChannelID, ChannelValue),
    ChannelRemove(ChannelID),

    Message(ChannelID, MessageValueRequest),
    MessageModify(MessageID, MessageValueRequest),
    MessageRemove(MessageID),
    // TO-DO merge with MessageValueRequest?
    MessageReply(MessageID, MessageValueRequest),
    MessageReact(MessageID),
    MessageStar(MessageID),

    Emote(EmoteValueRequest),
    EmoteRemove(EmoteID),

    Stamp(StampValueRequest),
    StampRemove(StampID),

    Role(RoleValue),
    RoleModify(RoleID, RoleValue),
    RoleRemove(RoleID),

    Invite(InviteValue),
    InviteRemove(InviteID),

    ViewAccount,
    ViewChannel,
    ViewMessage(ChannelID),
    ViewEmote,
    ViewStamp,
    ViewFile(FileID),
    ViewRole,
    ViewInvite,

    PollVote(MessageID, usize),

    AccountChannel(ChannelID),
    //AccountActivity(Option<AccountActivity>),
    AccountPresence(AccountPresence),
    AccountState(Option<String>),
    AccountWrite(bool),
}

impl CommandClient {
    pub async fn write(&self, socket: &mut TcpStream) {
        let data = serialize(self).unwrap();
        let size = (data.len() as u32).to_le_bytes();
        let mut size = vec![size[0], size[1], size[2], size[3]];
        size.extend(data);

        socket.write_all(&size).await.unwrap();
    }

    pub async fn read(socket: &mut TcpStream) -> anyhow::Result<Self> {
        let mut size = [0; 4];

        socket.read_exact(&mut size).await?;
        let size = u32::from_le_bytes(size);
        let mut buffer = vec![0; size as usize];

        socket.read_exact(&mut buffer).await?;
        let command = deserialize(&buffer)?;

        Ok(command)
    }
}

//================================================================

pub type Challenge = Vec<u8>;

/// A command the server can send to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandServer {
    Error(Error),

    Enter(AccountID, Server),
    Leave,
    Nonce(Challenge),

    Configuration(Configuration),

    Channel(Channel),
    ChannelRemove(ChannelID),

    Message(Message),
    MessageRemove(MessageID),
    MessageEmbed(MessageID, MessageEmbed),

    Role(Role),
    RoleRemove(RoleID),

    Emote(Emote),
    EmoteRemove(EmoteID),

    Stamp(Stamp),
    StampRemove(StampID),

    Invite(Invite),
    InviteRemove(InviteID),

    ViewAccount(BTreeMap<AccountID, Account>),
    ViewChannel(BTreeMap<ChannelID, Channel>),
    ViewMessage(ChannelID, BTreeMap<MessageID, Message>),
    ViewEmote(BTreeMap<EmoteID, Emote>),
    ViewStamp(BTreeMap<StampID, Stamp>),
    ViewFile(FileID, File),
    ViewRole(BTreeMap<RoleID, Role>),
    ViewInvite(BTreeMap<InviteID, Invite>),

    PollVote(AccountID, MessageID, usize),

    AccountChannel(AccountID, ChannelID),
    //AccountActivity(AccountID, Option<AccountActivity>),
    AccountPresence(AccountID, AccountPresence),
    AccountState(AccountID, Option<String>),
    AccountWrite(AccountID, bool),
}

impl CommandServer {
    pub async fn write(&self, socket: &mut TcpStream) {
        let data = serialize(self).unwrap();
        let size = (data.len() as u32).to_le_bytes();
        let mut size = vec![size[0], size[1], size[2], size[3]];
        size.extend(data);

        socket.write_all(&size).await.unwrap();
    }

    pub async fn read(socket: &mut TcpStream) -> anyhow::Result<Self> {
        let mut size = [0; 4];

        socket.read_exact(&mut size).await?;
        let size = u32::from_le_bytes(size);
        let mut buffer = vec![0; size as usize];

        socket.read_exact(&mut buffer).await?;
        let command = deserialize(&buffer)?;

        Ok(command)
    }
}

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    Connect(ConnectError),
    Account(AccountError),
    Message(MessageError),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectError {
    TimeOut,
    Nonce,
}
