mod command;

//================================================================

use igd::Gateway;
use igd::{PortMappingProtocol, search_gateway};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;

//================================================================

use crate::command::*;
use nimbus_protocol::prelude::*;

//================================================================

struct App {
    client: HashMap<AccountID, (Account, tokio::sync::mpsc::Sender<CommandServer>)>,
    storage: Storage,
    argument: Argument,
}

impl App {
    fn new(argument: Argument) -> anyhow::Result<Self> {
        Ok(Self {
            client: Default::default(),
            storage: Storage::new(&argument.file)?,
            argument,
        })
    }
}

impl App {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    async fn send_all(&self, command: CommandServer) -> anyhow::Result<()> {
        for (_, (_, client)) in &self.client {
            client.send(command.clone()).await?;
        }

        Ok(())
    }

    async fn listen(
        app: Arc<Mutex<Self>>,
        socket: TcpListener,
        a_tx: Sender<(AccountID, CommandClient)>,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
                Ok((socket, _)) = socket.accept() => {
                    App::accept(app.clone(), socket, a_tx.clone());
                }
            }
        }

        Ok(())
    }

    fn accept(
        app: Arc<Mutex<Self>>,
        mut socket: TcpStream,
        a_tx: Sender<(AccountID, CommandClient)>,
    ) {
        let app = app.clone();
        let a_tx = a_tx.clone();
        let (tx, rx) = channel::<CommandServer>(128);

        println!("[SERVER::Accept] Connect: {socket:#?}");

        tokio::spawn(async move {
            loop {
                let command = CommandClient::read(&mut socket).await;

                println!("[SERVER::Accept] {command:#?}");

                if let Ok(command) = command {
                    match command {
                        CommandClient::Enter(account) => {
                            let mut app = app.lock().await;

                            if let Ok(Some(index)) = app.storage.get_account_key(account.key) {
                                app.client.insert(index, (account.into_account(index), tx));

                                CommandServer::Enter(index, Server::from_storage(&app.storage)?)
                                    .write(&mut socket)
                                    .await;

                                App::client(socket, a_tx, rx, index).await;
                                break;
                            } else {
                                let index = app.storage.count_account()?;
                                app.storage.insert_account_key(account.key, index)?;
                                app.storage
                                    .insert_account(account.clone().into_account(index))?;

                                app.client.insert(index, (account.into_account(index), tx));

                                CommandServer::Enter(index, Server::from_storage(&app.storage)?)
                                    .write(&mut socket)
                                    .await;

                                App::client(socket, a_tx, rx, index).await;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    fn handle(app: Arc<Mutex<Self>>, mut rx: Receiver<(AccountID, CommandClient)>) {
        tokio::spawn(async move {
            while let Some((index, command)) = rx.recv().await {
                println!("[SERVER::Loop] {command:#?}");

                match &command {
                    CommandClient::Channel(channel) => {
                        let app = app.lock().await;
                        let channel = Channel::new(app.storage.count_channel()?, channel.clone());

                        app.storage.insert_channel(channel.index, channel.clone())?;
                        app.send_all(CommandServer::Channel(channel)).await?;
                    }
                    CommandClient::ChannelModify(identifier, channel) => {
                        let app = app.lock().await;
                        let channel = Channel::new(*identifier, channel.clone());

                        app.storage.change_channel(*identifier, channel.clone())?;
                        app.send_all(CommandServer::Channel(channel)).await?;
                    }
                    CommandClient::ChannelRemove(identifier) => {
                        let app = app.lock().await;

                        app.storage.remove_channel(*identifier)?;
                        app.send_all(CommandServer::ChannelRemove(*identifier))
                            .await?;
                    }
                    CommandClient::Message(channel, message) => {
                        let mut app_l = app.lock().await;
                        let value =
                            MessageValue::from_request(message.clone(), &mut app_l.storage)?;
                        let message = Message::new(
                            app_l.storage.count_message(*channel)?,
                            Some(index),
                            value,
                            None,
                        )?;

                        if let MessageValue::Text(text) = &message.value
                            && let Ok(list) = get_link_list(text)
                            && let Some(link) = list.first().cloned()
                        {
                            let app = app.clone();
                            let index = message.index;

                            tokio::spawn(async move {
                                // TO-DO persist embed to storage, use embed table for embed cache
                                let mut app_l = app.lock().await;
                                let embed = MessageEmbed::new(&link, &mut app_l.storage).await?;
                                app_l
                                    .send_all(CommandServer::MessageEmbed(index, embed))
                                    .await?;

                                Ok::<(), anyhow::Error>(())
                            });
                        }

                        app_l.storage.insert_message(
                            app_l.storage.count_message(*channel)?,
                            message.clone(),
                        )?;
                        app_l.send_all(CommandServer::Message(message)).await?;
                    }
                    CommandClient::MessageRemove(message) => {
                        let app = app.lock().await;

                        app.storage.remove_message(*message)?;
                        app.send_all(CommandServer::MessageRemove(*message)).await?;
                    }
                    CommandClient::Role(role) => {
                        let app = app.lock().await;
                        let role = Role::new(app.storage.count_role()?, role.clone());

                        app.storage.insert_role(role.index, role.clone())?;
                        app.send_all(CommandServer::Role(role)).await?;
                    }
                    CommandClient::RoleModify(identifier, role) => {
                        let app = app.lock().await;
                        let role = Role::new(*identifier, role.clone());

                        app.storage.change_role(*identifier, role.clone())?;
                        app.send_all(CommandServer::Role(role)).await?;
                    }
                    CommandClient::RoleRemove(identifier) => {
                        let app = app.lock().await;

                        app.storage.remove_role(*identifier)?;
                        app.send_all(CommandServer::RoleRemove(*identifier)).await?;
                    }
                    CommandClient::Stamp(stamp) => {
                        let mut app = app.lock().await;
                        let stamp = Stamp::new(
                            app.storage.count_stamp()?,
                            StampValue::from_request(stamp.clone(), &mut app.storage)?,
                        );

                        app.storage.insert_stamp(stamp.index, stamp.clone())?;
                        app.send_all(CommandServer::Stamp(stamp)).await?;
                    }
                    CommandClient::StampRemove(identifier) => {
                        let app = app.lock().await;

                        app.storage.remove_stamp(*identifier)?;
                        app.send_all(CommandServer::StampRemove(*identifier))
                            .await?;
                    }
                    CommandClient::Invite(invite) => {
                        let app = app.lock().await;
                        let invite = Invite::new(invite.clone());

                        app.storage
                            .insert_invite(invite.value.index.clone(), invite.clone())?;
                        app.send_all(CommandServer::Invite(invite)).await?;
                    }
                    CommandClient::InviteRemove(identifier) => {
                        let app = app.lock().await;

                        app.storage.remove_invite(identifier.clone())?;
                        app.send_all(CommandServer::InviteRemove(identifier.clone()))
                            .await?;
                    }
                    CommandClient::ViewAccount => {
                        let app = app.lock().await;
                        let view = app.storage.get_all_account()?;

                        app.send_all(CommandServer::ViewAccount(view)).await?;
                    }
                    CommandClient::ViewChannel => {
                        let app = app.lock().await;
                        let view = app.storage.get_all_channel()?;

                        app.send_all(CommandServer::ViewChannel(view)).await?;
                    }
                    CommandClient::ViewMessage(channel) => {
                        let app = app.lock().await;
                        let (_, count_max) = app.storage.count_message(*channel)?;
                        let count_min = count_max.saturating_sub(5);
                        let view = app
                            .storage
                            .get_range_message((*channel, count_min)..(*channel, count_max))?;

                        app.send_all(CommandServer::ViewMessage(*channel, view))
                            .await?;
                    }
                    CommandClient::ViewEmote => {
                        let app = app.lock().await;
                        let view = app.storage.get_all_emote()?;

                        app.send_all(CommandServer::ViewEmote(view)).await?;
                    }
                    CommandClient::ViewStamp => {
                        let app = app.lock().await;
                        let view = app.storage.get_all_stamp()?;

                        app.send_all(CommandServer::ViewStamp(view)).await?;
                    }
                    CommandClient::ViewFile(identifier) => {
                        let app = app.lock().await;

                        if let Some(file) = app.storage.get_file(*identifier)? {
                            app.send_all(CommandServer::ViewFile(*identifier, file))
                                .await?;
                        }
                    }
                    CommandClient::ViewRole => {
                        let app = app.lock().await;
                        let view = app.storage.get_all_role()?;

                        app.send_all(CommandServer::ViewRole(view)).await?;
                    }
                    CommandClient::ViewInvite => {
                        let app = app.lock().await;
                        let view = app.storage.get_all_invite()?;

                        app.send_all(CommandServer::ViewInvite(view)).await?;
                    }
                    CommandClient::PollVote(message, choice) => {
                        let app = app.lock().await;

                        app.storage.edit_message(*message, |message| {
                            if let MessageValue::Poll(poll) = &mut message.value {
                                let entry = poll.vote.entry(*choice).or_default();
                                if entry.contains(&index) {
                                    entry.remove(&index);
                                } else {
                                    entry.insert(index);
                                }
                            }
                        })?;
                        app.send_all(CommandServer::PollVote(index, *message, *choice))
                            .await?;
                    }
                    CommandClient::AccountChannel(channel) => {
                        let app = app.lock().await;

                        app.storage.edit_account(index, |account| {
                            account.channel = *channel;
                        })?;
                        app.send_all(CommandServer::AccountChannel(index, *channel))
                            .await?;
                    }
                    CommandClient::AccountPresence(presence) => {
                        let app = app.lock().await;

                        app.storage.edit_account(index, |account| {
                            account.presence = presence.clone();
                        })?;
                        app.send_all(CommandServer::AccountPresence(index, presence.clone()))
                            .await?;
                    }
                    CommandClient::AccountState(state) => {
                        let app = app.lock().await;

                        app.storage.edit_account(index, |account| {
                            account.state = state.clone();
                        })?;
                        app.send_all(CommandServer::AccountState(index, state.clone()))
                            .await?;
                    }
                    CommandClient::AccountWrite(write) => {
                        let app = app.lock().await;

                        app.storage.edit_account(index, |account| {
                            account.write = *write;
                        })?;
                        app.send_all(CommandServer::AccountWrite(index, *write))
                            .await?;
                    }
                    _ => {}
                }
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    async fn client(
        mut handle: TcpStream,
        tx: tokio::sync::mpsc::Sender<(AccountID, CommandClient)>,
        mut rx: tokio::sync::mpsc::Receiver<CommandServer>,
        account: AccountID,
    ) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = CommandClient::read(&mut handle) => {
                        match result {
                            Ok(command) => {
                                tx.send((account, command)).await?;
                            }
                            Err(err) => {
                                eprintln!("[SERVER] read error: {err}");
                                tx.send((account, CommandClient::Leave)).await?;
                                break;
                            }
                        }
                    }
                    Some(command) = rx.recv() => {
                        //println!("[SERVER] wrote command: {command:?}");
                        command.write(&mut handle).await;
                    }
                }
            }

            Ok::<(), anyhow::Error>(())
        });
    }

    async fn socket(app: Arc<Mutex<Self>>) -> anyhow::Result<TcpListener> {
        Ok(TcpListener::bind(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            app.lock().await.argument.port,
        ))
        .await?)
    }

    fn gateway() -> anyhow::Result<Gateway> {
        let gateway = search_gateway(Default::default())?;
        let address = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 6), 8080);
        gateway.add_port(PortMappingProtocol::TCP, 8080, address, 0, "Nimbus Server")?;
        Ok(gateway)
    }

    fn welcome(&self) {
        println!(
            "Welcome to Nimbus! Have a nimble day.\n  • Version: {}\n  • File: {}\n  • Port: {}\n  • uPnP status: {}",
            Self::VERSION,
            self.argument.file,
            self.argument.port,
            if self.argument.uPnP {
                "active"
            } else {
                "inactive"
            }
        );
    }
}

impl Drop for App {
    fn drop(&mut self) {
        /*
        if let Some(gateway) = &self.uPnP
            && gateway.remove_port(PortMappingProtocol::TCP, 8080).is_err()
        {
            println!("error: could not remove uPnP port.");
        }
        */
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    nimbus_protocol::utility::set_panic_hook("server");

    match Command::new() {
        Command::Run(argument) => {
            let app = Arc::new(Mutex::new(App::new(argument)?));
            let socket = App::socket(app.clone()).await?;
            let (tx, rx) = channel::<(AccountID, CommandClient)>(256);

            app.lock().await.welcome();

            App::handle(app.clone(), rx);
            App::listen(app, socket, tx).await?;

            Ok(())
        }
        Command::Help => {
            println!("{}", Command::HELP_TEXT.replace("{version}", App::VERSION));

            Ok(())
        }
    }
}
