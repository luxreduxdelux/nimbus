use redb::ReadableTable;
use redb::{Database, ReadableDatabase, TableDefinition};
use redb::{ReadTransaction, WriteTransaction};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

//================================================================

use crate::account::*;
use crate::channel::*;
use crate::message::*;
use crate::utility::*;

//================================================================

type Table<'a> = TableDefinition<'a, u64, Vec<u8>>;

pub struct Storage {
    data: Database,
}

impl Storage {
    const TABLE_META: TableDefinition<'_, u64, u64> = TableDefinition::new("meta");
    const TABLE_ACCOUNT_KEY: TableDefinition<'_, AccountKey, AccountID> =
        TableDefinition::new("account_key");
    const TABLE_ACCOUNT: Table<'_> = TableDefinition::new("account");
    const TABLE_CHANNEL: Table<'_> = TableDefinition::new("channel");
    const TABLE_MESSAGE: Table<'_> = TableDefinition::new("message");
    const INDEX_COUNT_ACCOUNT: u64 = 0;
    const INDEX_COUNT_CHANNEL: u64 = 1;
    const INDEX_COUNT_MESSAGE: u64 = 2;

    //================================================================

    pub fn new(path: &str) -> anyhow::Result<Self> {
        let exist = std::fs::exists(path).unwrap_or_default();
        let data = Database::create(path)?;
        let this = Self { data };

        if !exist {
            this.insert_channel(Channel::new(
                this.count_channel()?,
                Channel::DEFAULT_NAME.to_string(),
                Channel::DEFAULT_INFO.to_string(),
            ))?;
        }

        Ok(this)
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
        self.insert(Self::TABLE_ACCOUNT, Self::INDEX_COUNT_ACCOUNT, account)
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

    pub fn count_account(&self) -> anyhow::Result<u64> {
        self.get_meta_index(Self::INDEX_COUNT_ACCOUNT)
    }

    //================================================================

    pub fn insert_channel(&self, channel: Channel) -> anyhow::Result<()> {
        self.insert(Self::TABLE_CHANNEL, Self::INDEX_COUNT_CHANNEL, channel)
    }

    pub fn get_all_channel(&self) -> anyhow::Result<BTreeMap<ChannelID, Channel>> {
        self.get_all(Self::TABLE_CHANNEL)
    }

    pub fn count_channel(&self) -> anyhow::Result<u64> {
        self.get_meta_index(Self::INDEX_COUNT_CHANNEL)
    }

    //================================================================

    pub fn insert_message(&self, message: Message) -> anyhow::Result<()> {
        self.insert(Self::TABLE_MESSAGE, Self::INDEX_COUNT_MESSAGE, message)
    }

    pub fn remove_message(&self, message: MessageID) -> anyhow::Result<()> {
        self.remove(Self::TABLE_MESSAGE, message)
    }

    pub fn get_all_message(&self) -> anyhow::Result<BTreeMap<ChannelID, Message>> {
        self.get_all(Self::TABLE_MESSAGE)
    }

    pub fn count_message(&self) -> anyhow::Result<u64> {
        self.get_meta_index(Self::INDEX_COUNT_MESSAGE)
    }

    //================================================================

    fn get_all<V: for<'a> Deserialize<'a>>(
        &self,
        table: TableDefinition<u64, Vec<u8>>,
    ) -> anyhow::Result<BTreeMap<u64, V>> {
        self.read(|txn| {
            if let Ok(table) = txn.open_table(table) {
                let map: Result<BTreeMap<u64, V>, anyhow::Error> = table
                    .iter()?
                    .map(|entry| {
                        let (key, value) = entry?;
                        Ok((key.value(), deserialize(&value.value())?))
                    })
                    .collect();

                let map = map?;

                Ok(map)
            } else {
                Ok(BTreeMap::default())
            }
        })
    }

    fn get_meta_index(&self, key: u64) -> anyhow::Result<u64> {
        let read = self.read(|txn| {
            if let Ok(table) = txn.open_table(Self::TABLE_META) {
                Ok(table.get(key)?.map(|x| x.value()))
            } else {
                Ok(Some(0))
            }
        })?;

        Ok(read.unwrap_or_default())
    }

    fn get_meta_index_increment(&self, key: u64) -> anyhow::Result<u64> {
        let read = self.write(|txn| {
            let mut table = txn.open_table(Self::TABLE_META)?;
            let in_table = table.get(key)?.is_some();

            if in_table {
                let value = table.get(key)?.unwrap().value() + 1;
                table.insert(key, value)?;
                Ok(value)
            } else {
                table.insert(key, 0)?;
                Ok(0)
            }
        })?;

        Ok(read)
    }

    fn insert<T: Serialize>(&self, table: Table, index: u64, value: T) -> anyhow::Result<()> {
        let key = self.get_meta_index_increment(index)?;
        self.write(|txn| {
            let mut table = txn.open_table(table)?;
            table.insert(key, serialize(&value)?)?;
            Ok(())
        })?;

        Ok(())
    }

    fn remove(&self, table: Table, index: u64) -> anyhow::Result<()> {
        self.write(|txn| {
            let mut table = txn.open_table(table)?;
            table.remove(index)?;
            Ok(())
        })?;

        Ok(())
    }

    fn edit<F: Fn(&mut T), T: Serialize + for<'a> Deserialize<'a>>(
        &self,
        table: Table,
        index: u64,
        call: F,
    ) -> anyhow::Result<()> {
        let key = self.get_meta_index_increment(index)?;
        self.write(|txn| {
            let mut table = txn.open_table(table)?;

            let value = if let Some(value) = table.get(key)? {
                let mut value = deserialize(&value.value())?;
                call(&mut value);
                Some(serialize(&value)?)
            } else {
                None
            };

            if let Some(value) = value {
                table.insert(key, value)?;
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
