use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::Hash;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::client::*;
use crate::command::*;
use crate::emote::*;
use crate::file::*;
use crate::invite::*;
use crate::message::*;
use crate::role::*;
use crate::stamp::*;

//================================================================

pub struct Cache {
    thread: ThreadTX,
    view_account: View<AccountID, BTreeMap<AccountID, Account>>,
    view_channel: View<ChannelID, BTreeMap<ChannelID, Channel>>,
    view_message: View<ChannelID, HashMap<ChannelID, BTreeMap<MessageID, Message>>>,
    view_emote: View<EmoteID, BTreeMap<EmoteID, Emote>>,
    view_stamp: View<StampID, BTreeMap<StampID, Stamp>>,
    view_file: View<FileID, BTreeMap<FileID, File>>,
    view_role: View<RoleID, BTreeMap<RoleID, Role>>,
    view_invite: View<InviteID, BTreeMap<InviteID, Invite>>,
}

impl Cache {
    pub fn new(thread: ThreadTX) -> Self {
        Self {
            thread,
            view_account: Default::default(),
            view_channel: Default::default(),
            view_message: Default::default(),
            view_emote: Default::default(),
            view_stamp: Default::default(),
            view_file: Default::default(),
            view_role: Default::default(),
            view_invite: Default::default(),
        }
    }

    pub fn update(&mut self, command: &CommandServer) {
        match command {
            CommandServer::Channel(data) => {
                self.view_channel.value.insert(data.index, data.clone());
            }
            CommandServer::ChannelRemove(index) => {
                self.view_channel.value.remove(index);
            }
            CommandServer::Message(data) => {
                let entry = self.view_message.value.entry(data.index.0).or_default();
                entry.insert(data.index, data.clone());
            }
            CommandServer::MessageRemove(index) => {
                let entry = self.view_message.value.entry(index.0).or_default();
                entry.remove(index);
            }
            CommandServer::MessageEmbed(index, embed) => {
                let entry = self.view_message.value.entry(index.0).or_default();
                if let Some(message) = entry.get_mut(index) {
                    message.embed = Some(embed.clone());
                }
            }
            CommandServer::Role(data) => {
                self.view_role.value.insert(data.index, data.clone());
            }
            CommandServer::RoleRemove(index) => {
                self.view_role.value.remove(index);
            }
            CommandServer::Emote(data) => {
                self.view_emote.value.insert(data.index, data.clone());
            }
            CommandServer::EmoteRemove(index) => {
                self.view_emote.value.remove(index);
            }
            CommandServer::Stamp(data) => {
                self.view_stamp.value.insert(data.index, data.clone());
            }
            CommandServer::StampRemove(index) => {
                self.view_stamp.value.remove(index);
            }
            CommandServer::Invite(data) => {
                self.view_invite
                    .value
                    .insert(data.value.index.clone(), data.clone());
            }
            CommandServer::InviteRemove(index) => {
                self.view_invite.value.remove(index);
            }
            CommandServer::ViewAccount(data) => {
                self.view_account.value.extend(data.clone());
                self.view_account.set_state_all(ViewState::RequestDone);
            }
            CommandServer::ViewChannel(data) => {
                self.view_channel.value.extend(data.clone());
                self.view_channel.set_state_all(ViewState::RequestDone);
            }
            CommandServer::ViewMessage(index, data) => {
                let entry = self.view_message.value.entry(*index).or_default();
                entry.extend(data.clone());
                self.view_message
                    .set_state_map(*index, ViewState::RequestDone);
            }
            CommandServer::ViewEmote(data) => {
                self.view_emote.value.extend(data.clone());
                self.view_emote.set_state_all(ViewState::RequestDone);
            }
            CommandServer::ViewStamp(data) => {
                self.view_stamp.value.extend(data.clone());
                self.view_stamp.set_state_all(ViewState::RequestDone);
            }
            CommandServer::ViewFile(index, data) => {
                self.view_file.value.insert(*index, data.clone());
                self.view_file.set_state_map(*index, ViewState::RequestDone);
            }
            CommandServer::ViewRole(data) => {
                self.view_role.value.extend(data.clone());
                self.view_role.set_state_all(ViewState::RequestDone);
            }
            CommandServer::ViewInvite(data) => {
                self.view_invite.value.extend(data.clone());
                self.view_invite.set_state_all(ViewState::RequestDone);
            }
            _ => {}
        }
    }

    //================================================================

    pub fn get_view_account(&mut self) -> Option<&BTreeMap<AccountID, Account>> {
        self.view_account
            .get(&self.thread, None, CommandClient::ViewAccount)
    }

    pub fn get_view_channel(&mut self) -> Option<&BTreeMap<ChannelID, Channel>> {
        self.view_channel
            .get(&self.thread, None, CommandClient::ViewChannel)
    }

    pub fn get_view_message(
        &mut self,
        channel: ChannelID,
    ) -> Option<&BTreeMap<MessageID, Message>> {
        if let Some(view) = self.view_message.get(
            &self.thread,
            Some(channel),
            CommandClient::ViewMessage(channel),
        ) {
            view.get(&channel)
        } else {
            None
        }
    }

    pub fn get_view_emote(&mut self) -> Option<&BTreeMap<EmoteID, Emote>> {
        self.view_emote
            .get(&self.thread, None, CommandClient::ViewEmote)
    }

    pub fn get_view_stamp(&mut self) -> Option<&BTreeMap<StampID, Stamp>> {
        self.view_stamp
            .get(&self.thread, None, CommandClient::ViewStamp)
    }

    pub fn get_view_role(&mut self) -> Option<&BTreeMap<RoleID, Role>> {
        self.view_role
            .get(&self.thread, None, CommandClient::ViewRole)
    }

    pub fn get_view_invite(&mut self) -> Option<&BTreeMap<InviteID, Invite>> {
        self.view_invite
            .get(&self.thread, None, CommandClient::ViewInvite)
    }

    //================================================================

    pub fn get_account(&mut self, account: AccountID) -> Option<&Account> {
        if let Some(view) = self.get_view_account() {
            view.get(&account)
        } else {
            None
        }
    }

    pub fn get_channel(&mut self, channel: ChannelID) -> Option<&Channel> {
        if let Some(view) = self.get_view_channel() {
            view.get(&channel)
        } else {
            None
        }
    }

    pub fn get_message(&mut self, message: MessageID) -> Option<&Message> {
        if let Some(view) = self.get_view_message(message.0) {
            view.get(&message)
        } else {
            None
        }
    }

    pub fn get_emote(&mut self, emote: EmoteID) -> Option<&Emote> {
        if let Some(view) = self.get_view_emote() {
            view.get(&emote)
        } else {
            None
        }
    }

    pub fn get_stamp(&mut self, stamp: StampID) -> Option<&Stamp> {
        if let Some(view) = self.get_view_stamp() {
            view.get(&stamp)
        } else {
            None
        }
    }

    pub fn get_file(&mut self, file: FileID) -> Option<&File> {
        if let Some(view) =
            self.view_file
                .get(&self.thread, Some(file), CommandClient::ViewFile(file))
        {
            view.get(&file)
        } else {
            None
        }
    }

    pub fn get_file_state(&mut self, file: FileID) -> ViewState {
        *self.view_file.state_map.entry(file).or_default()
    }

    pub fn get_role(&mut self, role: RoleID) -> Option<&Role> {
        if let Some(view) = self.get_view_role() {
            view.get(&role)
        } else {
            None
        }
    }
}

//================================================================

#[derive(Default)]
struct View<K, T> {
    value: T,
    state_all: ViewState,
    state_map: HashMap<K, ViewState>,
}

impl<K: Hash + Clone + Eq, T> View<K, T> {
    fn get(&mut self, thread: &ThreadTX, index: Option<K>, command: CommandClient) -> Option<&T> {
        if let Some(index) = &index {
            match self.get_state_map(index.clone()) {
                ViewState::None => {
                    thread.send(command);
                    self.set_state_map(index.clone(), ViewState::RequestMade);
                    None
                }
                ViewState::RequestDone => Some(&self.value),
                _ => None,
            }
        } else {
            match self.get_state_all() {
                ViewState::None => {
                    thread.send(command);
                    self.set_state_all(ViewState::RequestMade);
                    None
                }
                ViewState::RequestDone => Some(&self.value),
                _ => None,
            }
        }
    }

    fn get_mutable(
        &mut self,
        thread: &ThreadTX,
        index: Option<K>,
        command: CommandClient,
    ) -> Option<&mut T> {
        if let Some(index) = &index {
            match self.get_state_map(index.clone()) {
                ViewState::None => {
                    thread.send(command);
                    self.set_state_map(index.clone(), ViewState::RequestMade);
                    None
                }
                ViewState::RequestDone => Some(&mut self.value),
                _ => None,
            }
        } else {
            match self.get_state_all() {
                ViewState::None => {
                    thread.send(command);
                    self.set_state_all(ViewState::RequestMade);
                    None
                }
                ViewState::RequestDone => Some(&mut self.value),
                _ => None,
            }
        }
    }

    fn get_state_map(&mut self, index: K) -> &ViewState {
        self.state_map.entry(index).or_default()
    }

    fn set_state_map(&mut self, index: K, value: ViewState) {
        self.state_map.insert(index, value);
    }

    fn get_state_all(&self) -> &ViewState {
        &self.state_all
    }

    fn set_state_all(&mut self, value: ViewState) {
        self.state_all = value;
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum ViewState {
    #[default]
    None,
    RequestMade,
    RequestDone,
}
