use redb::{Database, ReadableDatabase, TableDefinition};
use redb::{Key, ReadableTable};
use redb::{ReadTransaction, WriteTransaction};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::RangeBounds;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::message::*;
use crate::utility::*;

//================================================================

type Table<'a, K> = TableDefinition<'a, K, Vec<u8>>;

#[derive(Default, Serialize, Deserialize)]
struct Meta {
    count_account: u64,
    count_channel: u64,
    count_message: HashMap<ChannelID, u64>,
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

    //================================================================

    pub fn new(path: &str) -> anyhow::Result<Self> {
        //std::fs::remove_file(path);

        let exist = std::fs::exists(path).unwrap_or_default();
        let data = Database::create(path)?;
        let this = Self { data };

        if !exist {
            this.create_meta()?;
            this.insert_channel(Channel::new(
                this.count_channel()?,
                "foo".to_string(),
                Channel::DEFAULT_INFO.to_string(),
            ))?;
            this.insert_channel(Channel::new(
                this.count_channel()?,
                "bar".to_string(),
                Channel::DEFAULT_INFO.to_string(),
            ))?;
            this.insert_channel(Channel::new(
                this.count_channel()?,
                "baz".to_string(),
                Channel::DEFAULT_INFO.to_string(),
            ))?;
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

    pub fn insert_channel(&self, channel: Channel) -> anyhow::Result<()> {
        self.insert(Self::TABLE_CHANNEL, self.count_channel()?, channel)?;
        self.edit_meta(|meta| meta.count_channel += 1)
    }

    pub fn get_all_channel(&self) -> anyhow::Result<BTreeMap<ChannelID, Channel>> {
        self.get_all(Self::TABLE_CHANNEL)
    }

    pub fn count_channel(&self) -> anyhow::Result<u64> {
        Ok(self.get_meta()?.count_channel)
    }

    //================================================================

    pub fn insert_message(&self, message: Message) -> anyhow::Result<()> {
        let channel = message.index;
        self.insert(Self::TABLE_MESSAGE, message.index, message)?;
        self.edit_meta(|meta| {
            let meta = meta.count_message.entry(channel.0).or_default();
            *meta += 1;
        })
    }

    pub fn remove_message(&self, message: MessageID) -> anyhow::Result<()> {
        self.remove(Self::TABLE_MESSAGE, message)
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

    fn insert<K: Key + Copy + Borrow<K::SelfType<'static>>, T: Serialize>(
        &self,
        table: Table<K>,
        index: K,
        value: T,
    ) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(table)?;
            table.insert(index, serialize(&value)?)?;
            Ok(())
        })?;

        Ok(())
    }

    fn remove<K: Key + Copy + Borrow<K::SelfType<'static>>>(
        &self,
        table: Table<K>,
        index: K,
    ) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(table)?;
            table.remove(index)?;
            Ok(())
        })?;

        Ok(())
    }

    fn get<K: Key + Copy + Borrow<K::SelfType<'static>>, V: for<'a> Deserialize<'a>>(
        &self,
        table: Table<K>,
        index: K,
    ) -> anyhow::Result<Option<V>> {
        self.read(|txn| {
            if let Ok(table) = txn.open_table(table) {
                if let Some(value) = table.get(index)? {
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
        K: Key + Copy + Borrow<K::SelfType<'static>>,
        V: Serialize + for<'a> Deserialize<'a>,
    >(
        &self,
        table: Table<K>,
        index: K,
        call: F,
    ) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(table)?;

            let value = if let Some(value) = table.get(index)? {
                let mut value = deserialize(&value.value())?;
                call(&mut value);
                Some(serialize(&value)?)
            } else {
                None
            };

            if let Some(value) = value {
                table.insert(index, value)?;
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
