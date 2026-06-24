use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::message::*;
use crate::server::*;

//================================================================

pub type Signature = Vec<u8>;

/// A command the client can send to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandClient {
    Enter(AccountConnect),
    Leave,
    Nonce(Signature),

    Message(ChannelID, MessageKind),
    MessageReact(ChannelID, MessageID),
    MessageStar(ChannelID, MessageID),
    MessageEdit(ChannelID, MessageID, Message),
    MessageDelete(ChannelID, MessageID),

    AccountChannel(ChannelID),
    AccountState(AccountState),
    AccountWrite(bool),
}

impl CommandClient {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap()
    }

    pub async fn write(&self, socket: &mut TcpStream) {
        let data = self.serialize();
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
        let (command, _) = bincode::serde::decode_from_slice(&buffer, bincode::config::standard())?;

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

    Message(Message),
    MessageDelete(ChannelID, MessageID),

    AccountChannel(AccountID, ChannelID),
    AccountState(AccountID, AccountState),
    AccountWrite(AccountID, bool),
}

impl CommandServer {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).unwrap()
    }

    pub async fn write(&self, socket: &mut TcpStream) {
        let data = self.serialize();
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
        let (command, _) = bincode::serde::decode_from_slice(&buffer, bincode::config::standard())?;

        Ok(command)
    }
}

//================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    Connect,
    Account,
    Message,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Error::Connect => "Connect error",
            Error::Account => "Account error",
            Error::Message => "Message error",
        };

        formatter.write_str(string)
    }
}
