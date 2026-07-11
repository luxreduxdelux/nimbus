use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;

//================================================================

use crate::{account::*, channel::*, client::*, command::*, message::*, token::*};

//================================================================

pub struct Cache {
    channel: ThreadTX,
    message: BTreeMap<MessageID, MessageCache>,
    view_account: View<BTreeMap<AccountID, Account>>,
    view_channel: View<BTreeMap<ChannelID, Channel>>,
    view_message: HashMap<ChannelID, View<BTreeMap<MessageID, Message>>>,
}

impl Cache {
    pub fn new(channel: ThreadTX) -> Self {
        Self {
            channel,
            message: Default::default(),
            view_account: Default::default(),
            view_channel: Default::default(),
            view_message: Default::default(),
        }
    }

    pub fn update(&mut self, command: &CommandServer) {
        match command {
            CommandServer::Message(message) => {
                self.message
                    .insert(message.index, MessageCache::new(message));
                let entry = self.view_message.entry(message.index.0).or_default();

                match &mut entry.state {
                    ViewState::None => {
                        // request message view here?
                    }
                    ViewState::RequestDone(list) => {
                        list.insert(message.index, message.clone());
                    }
                    _ => {
                        // we already made a request, don't step on it by changing to RequestMade
                    }
                };
            }
            CommandServer::ViewAccount(data) => self.view_account.set_done(data.clone()),
            CommandServer::ViewChannel(data) => self.view_channel.set_done(data.clone()),
            CommandServer::ViewMessage(channel, data) => {
                let entry = self.view_message.entry(*channel).or_default();
                entry.set_done(data.clone());
            }
            _ => {}
        }
    }

    //================

    pub fn get_view_account(&mut self) -> Option<&BTreeMap<AccountID, Account>> {
        self.view_account
            .get(&self.channel, CommandClient::ViewAccount)
    }

    pub fn get_view_channel(&mut self) -> Option<&BTreeMap<ChannelID, Channel>> {
        self.view_channel
            .get(&self.channel, CommandClient::ViewChannel)
    }

    pub fn get_view_message(
        &mut self,
        channel: ChannelID,
    ) -> Option<&BTreeMap<MessageID, Message>> {
        let view = self.view_message.entry(channel).or_default();
        view.get(&self.channel, CommandClient::ViewMessage(channel))
    }

    //================

    pub fn get_account(&mut self, account: AccountID) -> Option<&Account> {
        if let Some(view) = self.get_view_account()
            && let Some(account) = view.get(&account)
        {
            return Some(account);
        }

        None
    }

    pub fn get_channel(&mut self, channel: ChannelID) -> Option<&Channel> {
        if let Some(view) = self.get_view_channel()
            && let Some(channel) = view.get(&channel)
        {
            return Some(channel);
        }

        None
    }

    pub fn get_message(&mut self, channel: ChannelID, message: MessageID) -> Option<&Message> {
        if let Some(view) = self.get_view_message(channel)
            && let Some(message) = view.get(&message)
        {
            return Some(message);
        }

        None
    }
}

//================================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageCache {
    pub token: Vec<Token>,
    // TO-DO keep separate account mention HashSet?
}

impl MessageCache {
    fn new(message: &Message) -> Self {
        let token = if let MessageKind::Text(text) = &message.kind {
            let (token, _, _) = Token::parse(text);
            token
        } else {
            Vec::default()
        };

        Self { token }
    }
}

//================================================================

#[derive(Default)]
struct View<T> {
    state: ViewState<T>,
}

impl<T> View<T> {
    fn get(&mut self, channel: &ThreadTX, command: CommandClient) -> Option<&T> {
        if let ViewState::None = &self.state {
            channel.send(command);
            self.state = ViewState::RequestMade;
        }
        if let ViewState::RequestDone(data) = &self.state {
            return Some(data);
        }

        None
    }

    fn set_done(&mut self, data: T) {
        self.state = ViewState::RequestDone(data);
    }
}

#[derive(Default)]
enum ViewState<T> {
    #[default]
    None,
    RequestMade,
    RequestDone(T),
}
