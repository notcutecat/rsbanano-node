use rsnano_core::{Block, PendingKey};
use rsnano_ledger::DEV_GENESIS_ACCOUNT;

use crate::ledger::ledger_tests::{setup_legacy_receive_block, LedgerContext};

#[test]
fn clear_successor() {
    let ctx = LedgerContext::empty();
    let mut txn = ctx.ledger.rw_txn();
    let receive = setup_legacy_receive_block(&ctx, txn.as_mut());

    ctx.ledger
        .rollback(txn.as_mut(), &receive.receive_block.hash())
        .unwrap();

    assert_eq!(
        ctx.ledger
            .store
            .block()
            .successor(txn.txn(), &receive.open_block.hash()),
        None
    );
}

#[test]
fn rollback_frontiers() {
    let ctx = LedgerContext::empty();
    let mut txn = ctx.ledger.rw_txn();
    let receive = setup_legacy_receive_block(&ctx, txn.as_mut());

    ctx.ledger
        .rollback(txn.as_mut(), &receive.receive_block.hash())
        .unwrap();

    assert_eq!(
        ctx.ledger
            .get_frontier(txn.txn(), &receive.open_block.hash()),
        Some(receive.destination.account())
    );
    assert_eq!(
        ctx.ledger
            .get_frontier(txn.txn(), &receive.receive_block.hash()),
        None
    );
}

#[test]
fn update_account_info() {
    let ctx = LedgerContext::empty();
    let mut txn = ctx.ledger.rw_txn();
    let receive = setup_legacy_receive_block(&ctx, txn.as_mut());

    ctx.ledger
        .rollback(txn.as_mut(), &receive.receive_block.hash())
        .unwrap();

    let account_info = ctx
        .ledger
        .get_account_info(txn.txn(), &receive.destination.account())
        .unwrap();

    assert_eq!(account_info.head, receive.open_block.hash());
    assert_eq!(account_info.block_count, 1);
    assert_eq!(
        account_info.balance,
        receive.open_block.sideband().unwrap().balance
    );
}

#[test]
fn rollback_pending_info() {
    let ctx = LedgerContext::empty();
    let mut txn = ctx.ledger.rw_txn();
    let receive = setup_legacy_receive_block(&ctx, txn.as_mut());

    ctx.ledger
        .rollback(txn.as_mut(), &receive.receive_block.hash())
        .unwrap();

    let pending = ctx
        .ledger
        .get_pending(
            txn.txn(),
            &PendingKey::new(receive.destination.account(), receive.send_block.hash()),
        )
        .unwrap();

    assert_eq!(pending.source, *DEV_GENESIS_ACCOUNT);
    assert_eq!(pending.amount, receive.amount_received);
}

#[test]
fn rollback_vote_weight() {
    let ctx = LedgerContext::empty();
    let mut txn = ctx.ledger.rw_txn();
    let receive = setup_legacy_receive_block(&ctx, txn.as_mut());

    ctx.ledger
        .rollback(txn.as_mut(), &receive.receive_block.hash())
        .unwrap();

    assert_eq!(
        ctx.ledger.weight(&receive.destination.account()),
        receive.expected_balance - receive.amount_received
    );
}
