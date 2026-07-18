use redb::{
    Database, Key, ReadTransaction, ReadableDatabase, ReadableTable, TableDefinition,
    WriteTransaction,
};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::RangeBounds;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::configuration::*;
use crate::emote::*;
use crate::file::*;
use crate::invite::*;
use crate::message::*;
use crate::role::*;
use crate::stamp::*;
use crate::utility::*;

//================================================================

type Table<'a, K> = TableDefinition<'a, K, Vec<u8>>;

#[derive(Default, Serialize, Deserialize)]
struct Meta {
    configuration: Configuration,
    count_account: u64,
    count_channel: u64,
    count_message: HashMap<ChannelID, u64>,
    count_emote: u64,
    count_stamp: u64,
    count_file: u64,
    count_role: u64,
}

pub struct Storage {
    data: Database,
}

impl Storage {
    const TABLE_META: Table<'_, u64> = TableDefinition::new("meta");
    const TABLE_ACCOUNT_KEY: TableDefinition<'_, AccountKey, AccountID> =
        TableDefinition::new("account_key");
    const TABLE_ACCOUNT: Table<'_, AccountID> = TableDefinition::new("account");
    const TABLE_CHANNEL: Table<'_, ChannelID> = TableDefinition::new("channel");
    const TABLE_MESSAGE: Table<'_, MessageID> = TableDefinition::new("message");
    const TABLE_EMOTE: Table<'_, EmoteID> = TableDefinition::new("emote");
    const TABLE_STAMP: Table<'_, StampID> = TableDefinition::new("stamp");
    const TABLE_FILE: Table<'_, FileID> = TableDefinition::new("file");
    const TABLE_ROLE: Table<'_, RoleID> = TableDefinition::new("role");
    const TABLE_INVITE: Table<'_, InviteID> = TableDefinition::new("invite");

    //================================================================

    pub fn new(path: &str) -> anyhow::Result<Self> {
        std::fs::remove_file(path);

        let exist = std::fs::exists(path).unwrap_or_default();
        let data = Database::create(path)?;
        let this = Self { data };

        if !exist {
            this.create_meta()?;
            this.insert_channel(
                this.count_channel()?,
                Channel::new(
                    this.count_channel()?,
                    ChannelValue::new("foo".to_string(), ChannelValue::DEFAULT_INFO.to_string()),
                ),
            )?;
            this.insert_channel(
                this.count_channel()?,
                Channel::new(
                    this.count_channel()?,
                    ChannelValue::new("bar".to_string(), ChannelValue::DEFAULT_INFO.to_string()),
                ),
            )?;
            this.insert_channel(
                this.count_channel()?,
                Channel::new(
                    this.count_channel()?,
                    ChannelValue::new("baz".to_string(), ChannelValue::DEFAULT_INFO.to_string()),
                ),
            )?;
        }

        Ok(this)
    }

    fn create_meta(&self) -> anyhow::Result<()> {
        self.insert(Self::TABLE_META, 0, Meta::default())
    }

    fn get_meta(&self) -> anyhow::Result<Meta> {
        Ok(self.get(Self::TABLE_META, 0)?.unwrap_or_default())
    }

    fn edit_meta<F: Fn(&mut Meta)>(&self, call: F) -> anyhow::Result<()> {
        self.edit(Self::TABLE_META, 0, call)
    }

    //================================================================

    pub fn insert_account_key(&self, key: AccountKey, account: AccountID) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(Self::TABLE_ACCOUNT_KEY)?;
            table.insert(key, account)?;
            Ok(())
        })
    }

    pub fn get_account_key(&self, key: AccountKey) -> anyhow::Result<Option<AccountID>> {
        self.read(|txn| {
            let table = txn.open_table(Self::TABLE_ACCOUNT_KEY)?;
            Ok(table.get(key)?.map(|account| account.value()))
        })
    }

    //================================================================

    pub fn insert_account(&self, account: Account) -> anyhow::Result<()> {
        self.insert(Self::TABLE_ACCOUNT, self.count_account()?, account)?;
        self.edit_meta(|meta| meta.count_account += 1)
    }

    pub fn edit_account<F: Fn(&mut Account)>(
        &self,
        index: AccountID,
        call: F,
    ) -> anyhow::Result<()> {
        self.edit(Self::TABLE_ACCOUNT, index, call)
    }

    pub fn get_all_account(&self) -> anyhow::Result<BTreeMap<AccountID, Account>> {
        self.get_all(Self::TABLE_ACCOUNT)
    }

    pub fn count_account(&self) -> anyhow::Result<AccountID> {
        Ok(self.get_meta()?.count_account)
    }

    //================================================================

    pub fn insert_channel(&self, index: ChannelID, channel: Channel) -> anyhow::Result<()> {
        self.insert(Self::TABLE_CHANNEL, index, channel)?;
        self.edit_meta(|meta| meta.count_channel += 1)
    }

    pub fn change_channel(&self, index: ChannelID, channel: Channel) -> anyhow::Result<()> {
        if self.exist_channel(index)? {
            self.insert(Self::TABLE_CHANNEL, index, channel)?;
        }

        Ok(())
    }

    pub fn remove_channel(&self, index: ChannelID) -> anyhow::Result<()> {
        self.remove(Self::TABLE_CHANNEL, index)
    }

    pub fn exist_channel(&self, index: ChannelID) -> anyhow::Result<bool> {
        self.exist(Self::TABLE_CHANNEL, index)
    }

    pub fn get_all_channel(&self) -> anyhow::Result<BTreeMap<ChannelID, Channel>> {
        self.get_all(Self::TABLE_CHANNEL)
    }

    pub fn count_channel(&self) -> anyhow::Result<u64> {
        Ok(self.get_meta()?.count_channel)
    }

    //================================================================

    pub fn insert_message(&self, index: MessageID, message: Message) -> anyhow::Result<()> {
        self.insert(Self::TABLE_MESSAGE, index, message)?;
        self.edit_meta(|meta| {
            let meta = meta.count_message.entry(index.0).or_default();
            *meta += 1;
        })
    }

    pub fn remove_message(&self, index: MessageID) -> anyhow::Result<()> {
        self.remove(Self::TABLE_MESSAGE, index)
    }

    pub fn edit_message<F: Fn(&mut Message)>(
        &self,
        index: MessageID,
        call: F,
    ) -> anyhow::Result<()> {
        self.edit(Self::TABLE_MESSAGE, index, call)
    }

    pub fn get_range_message(
        &self,
        range: impl RangeBounds<MessageID> + Clone,
    ) -> anyhow::Result<BTreeMap<MessageID, Message>> {
        self.get_range(Self::TABLE_MESSAGE, range)
    }

    pub fn get_all_message(&self) -> anyhow::Result<BTreeMap<MessageID, Message>> {
        self.get_all(Self::TABLE_MESSAGE)
    }

    pub fn count_message(&self, channel: ChannelID) -> anyhow::Result<MessageID> {
        let mut meta = self.get_meta()?;
        let meta = meta.count_message.entry(channel).or_default();
        Ok((channel, *meta))
    }

    //================================================================

    pub fn insert_emote(&self, index: EmoteID, emote: Emote) -> anyhow::Result<()> {
        self.insert(Self::TABLE_EMOTE, index, emote)?;
        self.edit_meta(|meta| meta.count_emote += 1)
    }

    pub fn get_all_emote(&self) -> anyhow::Result<BTreeMap<EmoteID, Emote>> {
        self.get_all(Self::TABLE_EMOTE)
    }

    pub fn count_emote(&self) -> anyhow::Result<u64> {
        Ok(self.get_meta()?.count_emote)
    }

    //================================================================

    pub fn insert_stamp(&self, index: StampID, stamp: Stamp) -> anyhow::Result<()> {
        self.insert(Self::TABLE_STAMP, index, stamp)?;
        self.edit_meta(|meta| meta.count_stamp += 1)
    }

    pub fn get_all_stamp(&self) -> anyhow::Result<BTreeMap<StampID, Stamp>> {
        self.get_all(Self::TABLE_STAMP)
    }

    pub fn count_stamp(&self) -> anyhow::Result<u64> {
        Ok(self.get_meta()?.count_stamp)
    }

    //================================================================

    pub fn insert_file(&self, index: FileID, file: File) -> anyhow::Result<()> {
        self.insert(Self::TABLE_FILE, index, file)?;
        self.edit_meta(|meta| meta.count_file += 1)
    }

    pub fn get_file(&self, file: FileID) -> anyhow::Result<Option<File>> {
        self.get(Self::TABLE_FILE, file)
    }

    pub fn count_file(&self) -> anyhow::Result<u64> {
        Ok(self.get_meta()?.count_file)
    }

    //================================================================

    pub fn insert_role(&self, index: RoleID, role: Role) -> anyhow::Result<()> {
        self.insert(Self::TABLE_ROLE, index, role)?;
        self.edit_meta(|meta| meta.count_role += 1)
    }

    pub fn change_role(&self, index: RoleID, role: Role) -> anyhow::Result<()> {
        if self.exist_role(index)? {
            self.insert(Self::TABLE_ROLE, index, role)?;
        }

        Ok(())
    }

    pub fn remove_role(&self, index: RoleID) -> anyhow::Result<()> {
        self.remove(Self::TABLE_ROLE, index)
    }

    pub fn exist_role(&self, index: ChannelID) -> anyhow::Result<bool> {
        self.exist(Self::TABLE_ROLE, index)
    }

    pub fn get_all_role(&self) -> anyhow::Result<BTreeMap<RoleID, Role>> {
        self.get_all(Self::TABLE_ROLE)
    }

    pub fn count_role(&self) -> anyhow::Result<RoleID> {
        Ok(self.get_meta()?.count_role)
    }

    //================================================================

    pub fn insert_invite(&self, index: InviteID, invite: Invite) -> anyhow::Result<()> {
        self.insert(Self::TABLE_INVITE, index, invite)
    }

    pub fn remove_invite(&self, index: InviteID) -> anyhow::Result<()> {
        self.remove(Self::TABLE_INVITE, index)
    }

    pub fn get_all_invite(&self) -> anyhow::Result<BTreeMap<InviteID, Invite>> {
        self.get_all(Self::TABLE_INVITE)
    }

    //================================================================

    fn insert<K: Key + Clone + Borrow<K::SelfType<'static>>, T: Serialize>(
        &self,
        table: Table<K>,
        index: K,
        value: T,
    ) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(table)?;
            table.insert(index.clone(), serialize(&value)?)?;
            Ok(())
        })?;

        Ok(())
    }

    fn remove<K: Key + Clone + Borrow<K::SelfType<'static>>>(
        &self,
        table: Table<K>,
        index: K,
    ) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(table)?;
            table.remove(index.clone())?;
            Ok(())
        })?;

        Ok(())
    }

    fn exist<K: Key + Clone + Borrow<K::SelfType<'static>>>(
        &self,
        table: Table<K>,
        index: K,
    ) -> anyhow::Result<bool> {
        self.read(|txn| {
            if let Ok(table) = txn.open_table(table) {
                if let Some(_) = table.get(index.clone())? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        })
    }

    fn get<K: Key + Clone + Borrow<K::SelfType<'static>>, V: for<'a> Deserialize<'a>>(
        &self,
        table: Table<K>,
        index: K,
    ) -> anyhow::Result<Option<V>> {
        self.read(|txn| {
            if let Ok(table) = txn.open_table(table) {
                if let Some(value) = table.get(index.clone())? {
                    Ok(Some(deserialize(&value.value())?))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
    }

    fn get_range<
        K: Key + for<'a> From<K::SelfType<'a>> + Ord,
        V: for<'a> Deserialize<'a>,
        R: for<'a> RangeBounds<K::SelfType<'a>> + Clone,
    >(
        &self,
        table: Table<K>,
        range: R,
    ) -> anyhow::Result<BTreeMap<K, V>> {
        self.read(|txn| {
            if let Ok(table) = txn.open_table(table) {
                let map: Result<BTreeMap<K, V>, anyhow::Error> = table
                    .range(range.clone())?
                    .map(|entry| {
                        let (key, value) = entry?;
                        Ok((K::from(key.value()), deserialize(&value.value())?))
                    })
                    .collect();

                let map = map?;

                Ok(map)
            } else {
                Ok(BTreeMap::default())
            }
        })
    }

    fn get_all<K: Key + for<'a> From<K::SelfType<'a>> + Ord, V: for<'a> Deserialize<'a>>(
        &self,
        table: Table<K>,
    ) -> anyhow::Result<BTreeMap<K, V>> {
        self.read(|txn| {
            if let Ok(table) = txn.open_table(table) {
                let map: Result<BTreeMap<K, V>, anyhow::Error> = table
                    .iter()?
                    .map(|entry| {
                        let (key, value) = entry?;
                        Ok((K::from(key.value()), deserialize(&value.value())?))
                    })
                    .collect();

                let map = map?;

                Ok(map)
            } else {
                Ok(BTreeMap::default())
            }
        })
    }

    fn edit<
        F: Fn(&mut V),
        K: Key + Clone + Borrow<K::SelfType<'static>>,
        V: Serialize + for<'a> Deserialize<'a>,
    >(
        &self,
        table: Table<K>,
        index: K,
        call: F,
    ) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(table)?;

            let value = if let Some(value) = table.get(index.clone())? {
                let mut value = deserialize(&value.value())?;
                call(&mut value);
                Some(serialize(&value)?)
            } else {
                None
            };

            if let Some(value) = value {
                table.insert(index.clone(), value)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn write<F: Fn(&WriteTransaction) -> anyhow::Result<T>, T>(
        &self,
        call: F,
    ) -> anyhow::Result<T> {
        let txn = self.data.begin_write()?;
        let value = call(&txn)?;
        txn.commit()?;

        Ok(value)
    }

    fn read<F: Fn(&ReadTransaction) -> anyhow::Result<T>, T>(&self, call: F) -> anyhow::Result<T> {
        let txn = self.data.begin_read()?;
        call(&txn)
    }
}
