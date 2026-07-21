use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

//================================================================

pub type InviteID = String;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Invite {
    pub date: DateTime<Utc>,
    pub value: InviteValue,
}

impl Invite {
    // TO-DO should this be a constant in another struct?
    pub const DEFAULT_PORT: u16 = 8080;

    pub fn new(value: InviteValue) -> Self {
        Self {
            date: Utc::now(),
            value,
        }
    }

    pub fn parse(invite: &str) -> Result<(Option<String>, SocketAddr), InviteError> {
        let token: Vec<&str> = invite.split('@').collect();

        match token.len() {
            0 => Err(InviteError::Empty),
            1 => {
                let server = Self::resolve(token[0])?;

                Ok((None, server))
            }
            2 => {
                let invite = token[0];
                let server = Self::resolve(token[1])?;

                Ok((Some(invite.to_string()), server))
            }
            _ => Err(InviteError::InvalidSyntax),
        }
    }

    fn resolve(host: &str) -> Result<SocketAddr, InviteError> {
        if let Ok(mut address) = host.to_socket_addrs()
            && let Some(address) = address.next()
        {
            if address.port() == 0 {
                Ok(SocketAddr::new(address.ip(), Self::DEFAULT_PORT))
            } else {
                Ok(address)
            }
        } else {
            Err(InviteError::InvalidServer)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InviteError {
    Empty,
    InvalidInvite,
    InvalidServer,
    InvalidSyntax,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InviteValue {
    pub index: InviteID,
    pub count: Option<u64>,
    pub time: Duration,
}
