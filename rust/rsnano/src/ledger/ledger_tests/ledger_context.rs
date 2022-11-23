use std::{sync::Arc, time::Duration};

use crate::{
    config::TxnTrackingConfig,
    core::{Account, Amount, Block, BlockBuilder, BlockHash, KeyPair, Link, StateBlock},
    ledger::{
        datastore::{
            lmdb::{EnvOptions, LmdbStore, TestDbFile},
            WriteTransaction,
        },
        GenerateCache, Ledger,
    },
    stats::{Stat, StatConfig},
    utils::NullLogger,
    DEV_CONSTANTS, DEV_GENESIS_ACCOUNT,
};

pub(crate) struct LedgerContext {
    pub(crate) ledger: Ledger,
    db_file: TestDbFile,
}

impl LedgerContext {
    pub fn empty() -> Self {
        let db_file = TestDbFile::random();
        let store = Arc::new(
            LmdbStore::new(
                &db_file.path,
                &EnvOptions::default(),
                TxnTrackingConfig::default(),
                Duration::from_millis(5000),
                Arc::new(NullLogger::new()),
                false,
            )
            .unwrap(),
        );

        let ledger = Ledger::new(
            store.clone(),
            DEV_CONSTANTS.clone(),
            Arc::new(Stat::new(StatConfig::default())),
            &GenerateCache::new(),
        )
        .unwrap();

        let mut txn = store.tx_begin_write().unwrap();
        store.initialize(&mut txn, &ledger.cache, &DEV_CONSTANTS);

        LedgerContext { ledger, db_file }
    }

    pub(crate) fn process_state_receive(
        &self,
        txn: &mut dyn WriteTransaction,
        send: &dyn Block,
        receiver_key: &KeyPair,
    ) -> StateBlock {
        let receiver_account = receiver_key.public_key().into();
        let receiver_account_info = self
            .ledger
            .store
            .account()
            .get(txn.txn(), &receiver_account)
            .unwrap();

        let amount = self.ledger.amount(txn.txn(), &send.hash()).unwrap();

        let mut receive = BlockBuilder::state()
            .account(receiver_account)
            .previous(receiver_account_info.head)
            .balance(receiver_account_info.balance + amount)
            .representative(*DEV_GENESIS_ACCOUNT)
            .link(send.hash())
            .sign(&receiver_key)
            .build();

        self.ledger.process(txn, &mut receive).unwrap();

        receive
    }

    pub(crate) fn process_state_change(
        &self,
        txn: &mut dyn WriteTransaction,
        key: &KeyPair,
        rep_account: Account,
    ) -> StateBlock {
        let account = key.public_key().into();
        let account_info = self
            .ledger
            .store
            .account()
            .get(txn.txn(), &account)
            .unwrap();

        let mut change = BlockBuilder::state()
            .account(account)
            .previous(account_info.head)
            .representative(rep_account)
            .balance(account_info.balance)
            .link(Link::zero())
            .sign(key)
            .build();

        self.ledger.process(txn, &mut change).unwrap();
        change
    }

    pub(crate) fn process_state_open(
        &self,
        txn: &mut dyn WriteTransaction,
        send: &dyn Block,
        receiver_key: &KeyPair,
    ) -> StateBlock {
        let receiver_account: Account = receiver_key.public_key().into();
        let amount = self.ledger.amount(txn.txn(), &send.hash()).unwrap();

        let mut open_block = BlockBuilder::state()
            .account(receiver_account)
            .previous(BlockHash::zero())
            .balance(amount)
            .representative(receiver_account)
            .link(send.hash())
            .sign(&receiver_key)
            .build();

        self.ledger.process(txn, &mut open_block).unwrap();

        open_block
    }
}

pub(crate) struct SendStateBlockInfo {
    pub send_block: StateBlock,
    pub receiver_key: KeyPair,
    pub receiver_account: Account,
    pub amount_sent: Amount,
}
