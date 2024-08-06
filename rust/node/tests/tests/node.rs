use crate::tests::helpers::{
    assert_always_eq, assert_never, assert_timely, assert_timely_eq, make_fake_channel, System,
};
use rsnano_core::{
    utils::milliseconds_since_epoch, work::WorkPool, Amount, BlockEnum, BlockHash, KeyPair, RawKey,
    SendBlock, Signature, StateBlock, Vote, VoteSource, VoteWithWeightInfo, DEV_GENESIS_KEY,
};
use rsnano_ledger::{Writer, DEV_GENESIS_ACCOUNT, DEV_GENESIS_HASH};
use rsnano_messages::{ConfirmAck, DeserializedMessage, Message, Publish};
use rsnano_node::{
    config::NodeFlags,
    consensus::{ActiveElectionsExt, VoteApplierExt},
    stats::{DetailType, Direction, StatType},
    transport::{
        BufferDropPolicy, ChannelDirection, ChannelEnum, ChannelTcp, PeerConnectorExt, TcpStream,
        TrafficType,
    },
    wallets::WalletsExt,
};
use std::{sync::Arc, thread::sleep, time::Duration};
use tracing::error;

use super::helpers::start_election;

#[test]
fn local_block_broadcast() {
    let mut system = System::new();

    let mut node_config = System::default_config();
    node_config.priority_scheduler_enabled = false;
    node_config.hinted_scheduler.enabled = false;
    node_config.optimistic_scheduler.enabled = false;
    node_config.local_block_broadcaster.rebroadcast_interval = Duration::from_secs(1);

    let node1 = system.build_node().config(node_config).finish();
    let node2 = system.make_disconnected_node();

    let key1 = KeyPair::new();
    let latest_hash = *DEV_GENESIS_HASH;

    let send1 = BlockEnum::LegacySend(SendBlock::new(
        &latest_hash,
        &key1.public_key(),
        &(Amount::MAX - Amount::nano(1000)),
        &DEV_GENESIS_KEY.private_key(),
        &DEV_GENESIS_KEY.public_key(),
        system.work.generate_dev2(latest_hash.into()).unwrap(),
    ));

    let qualified_root = send1.qualified_root();
    let send_hash = send1.hash();
    node1.process_local(send1).unwrap();

    assert_never(Duration::from_millis(500), || {
        node1.active.active_root(&qualified_root)
    });

    // Wait until a broadcast is attempted
    assert_timely_eq(
        Duration::from_secs(5),
        || node1.local_block_broadcaster.len(),
        1,
    );
    assert_timely(
        Duration::from_secs(5),
        || {
            node1.stats.count(
                StatType::LocalBlockBroadcaster,
                DetailType::Broadcast,
                Direction::Out,
            ) >= 1
        },
        "no broadcast sent",
    );

    // The other node should not have received a block
    assert_never(Duration::from_millis(500), || {
        node2.block(&send_hash).is_some()
    });

    error!(
        "node2 local addr is {:?}",
        node2.tcp_listener.local_address()
    );

    // Connect the nodes and check that the block is propagated
    node1
        .peer_connector
        .connect_to(node2.tcp_listener.local_address());
    assert_timely(
        Duration::from_secs(5),
        || node1.network.find_node_id(&node2.get_node_id()).is_some(),
        "node2 not connected",
    );
    assert_timely(
        Duration::from_secs(10),
        || node2.block(&send_hash).is_some(),
        "block not received",
    )
}

#[test]
fn fork_no_vote_quorum() {
    let mut system = System::new();
    let node1 = system.make_node();
    let node2 = system.make_node();
    let node3 = system.make_node();
    let wallet_id1 = node1.wallets.wallet_ids()[0];
    let wallet_id2 = node2.wallets.wallet_ids()[0];
    let wallet_id3 = node3.wallets.wallet_ids()[0];
    node1
        .wallets
        .insert_adhoc2(&wallet_id1, &DEV_GENESIS_KEY.private_key(), true)
        .unwrap();
    let key4 = node1
        .wallets
        .deterministic_insert2(&wallet_id1, true)
        .unwrap();
    node1
        .wallets
        .send_action2(
            &wallet_id1,
            *DEV_GENESIS_ACCOUNT,
            key4,
            Amount::MAX / 4,
            0,
            true,
            None,
        )
        .unwrap();
    let key1 = node2
        .wallets
        .deterministic_insert2(&wallet_id2, true)
        .unwrap();
    node2
        .wallets
        .set_representative(wallet_id2, key1, false)
        .unwrap();
    let block = node1
        .wallets
        .send_action2(
            &wallet_id1,
            *DEV_GENESIS_ACCOUNT,
            key1,
            node1.config.receive_minimum,
            0,
            true,
            None,
        )
        .unwrap();
    assert_timely(
        Duration::from_secs(30),
        || {
            node3.balance(&key1) == node1.config.receive_minimum
                && node2.balance(&key1) == node1.config.receive_minimum
                && node1.balance(&key1) == node1.config.receive_minimum
        },
        "balances are wrong",
    );
    assert_eq!(node1.config.receive_minimum, node1.ledger.weight(&key1));
    assert_eq!(node1.config.receive_minimum, node2.ledger.weight(&key1));
    assert_eq!(node1.config.receive_minimum, node3.ledger.weight(&key1));

    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        block.hash(),
        *DEV_GENESIS_ACCOUNT,
        (Amount::MAX / 4) - (node1.config.receive_minimum * 2),
        key1.into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev(block.hash().into()),
    ));

    node1.process(send1.clone()).unwrap();
    node2.process(send1.clone()).unwrap();
    node3.process(send1.clone()).unwrap();

    let key2 = node3
        .wallets
        .deterministic_insert2(&wallet_id3, true)
        .unwrap();

    let send2 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        block.hash(),
        *DEV_GENESIS_ACCOUNT,
        (Amount::MAX / 4) - (node1.config.receive_minimum * 2),
        key2.into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev(block.hash().into()),
    ));
    let key3 = RawKey::random();
    let vote = Vote::new(key1, &key3, 0, 0, vec![send2.hash()]);
    let confirm = Message::ConfirmAck(ConfirmAck::new_with_own_vote(vote));
    let channel = node2
        .network
        .find_node_id(&node3.node_id.public_key())
        .unwrap();
    channel.try_send(
        &confirm,
        BufferDropPolicy::NoLimiterDrop,
        TrafficType::Generic,
    );

    assert_timely(
        Duration::from_secs(10),
        || {
            node3
                .stats
                .count(StatType::Message, DetailType::ConfirmAck, Direction::In)
                >= 3
        },
        "no confirm ack",
    );
    assert_eq!(node1.latest(&DEV_GENESIS_ACCOUNT), send1.hash());
    assert_eq!(node2.latest(&DEV_GENESIS_ACCOUNT), send1.hash());
    assert_eq!(node3.latest(&DEV_GENESIS_ACCOUNT), send1.hash());
}

#[test]
fn fork_open() {
    let mut system = System::new();
    let node = system.make_node();
    let wallet_id = node.wallets.wallet_ids()[0];

    // create block send1, to send all the balance from genesis to key1
    // this is done to ensure that the open block(s) cannot be voted on and confirmed
    let key1 = KeyPair::new();
    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::zero(),
        key1.public_key().into(),
        &DEV_GENESIS_KEY,
        node.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));

    let channel = make_fake_channel(&node);

    node.inbound_message_queue.put(
        DeserializedMessage::new(
            Message::Publish(Publish::new_forward(send1.clone())),
            node.network_params.network.protocol_info(),
        ),
        channel.clone(),
    );

    assert_timely(
        Duration::from_secs(5),
        || node.active.election(&send1.qualified_root()).is_some(),
        "election not found",
    );
    let election = node.active.election(&send1.qualified_root()).unwrap();
    node.active.force_confirm(&election);
    assert_timely_eq(Duration::from_secs(5), || node.active.len(), 0);

    // register key for genesis account, not sure why we do this, it seems needless,
    // since the genesis account at this stage has zero voting weight
    node.wallets
        .insert_adhoc2(&wallet_id, &DEV_GENESIS_KEY.private_key(), true)
        .unwrap();

    // create the 1st open block to receive send1, which should be regarded as the winner just because it is first
    let open1 = BlockEnum::State(StateBlock::new(
        key1.public_key(),
        BlockHash::zero(),
        1.into(),
        Amount::MAX,
        send1.hash().into(),
        &key1,
        node.work_generate_dev(key1.public_key().into()),
    ));
    node.inbound_message_queue.put(
        DeserializedMessage::new(
            Message::Publish(Publish::new_forward(open1.clone())),
            node.network_params.network.protocol_info(),
        ),
        channel.clone(),
    );
    assert_timely_eq(Duration::from_secs(5), || node.active.len(), 1);

    // create 2nd open block, which is a fork of open1 block
    // create the 1st open block to receive send1, which should be regarded as the winner just because it is first
    let open2 = BlockEnum::State(StateBlock::new(
        key1.public_key(),
        BlockHash::zero(),
        2.into(),
        Amount::MAX,
        send1.hash().into(),
        &key1,
        node.work_generate_dev(key1.public_key().into()),
    ));
    node.inbound_message_queue.put(
        DeserializedMessage::new(
            Message::Publish(Publish::new_forward(open2.clone())),
            node.network_params.network.protocol_info(),
        ),
        channel.clone(),
    );
    assert_timely(
        Duration::from_secs(5),
        || node.active.election(&open2.qualified_root()).is_some(),
        "no election for open2",
    );

    let election = node.active.election(&open2.qualified_root()).unwrap();
    // we expect to find 2 blocks in the election and we expect the first block to be the winner just because it was first
    assert_timely_eq(
        Duration::from_secs(5),
        || election.mutex.lock().unwrap().last_blocks.len(),
        2,
    );
    assert_eq!(
        open1.hash(),
        election
            .mutex
            .lock()
            .unwrap()
            .status
            .winner
            .as_ref()
            .unwrap()
            .hash()
    );

    // wait for a second and check that the election did not get confirmed
    sleep(Duration::from_millis(1000));
    assert_eq!(node.active.confirmed(&election), false);

    // check that only the first block is saved to the ledger
    assert_timely(
        Duration::from_secs(5),
        || node.block_exists(&open1.hash()),
        "open1 not in ledger",
    );
    assert_eq!(node.block_exists(&open2.hash()), false);
}

#[test]
fn online_reps_rep_crawler() {
    let mut system = System::new();
    let mut flags = NodeFlags::default();
    flags.disable_rep_crawler = true;
    let node = system.build_node().flags(flags).finish();
    let vote = Arc::new(Vote::new(
        *DEV_GENESIS_ACCOUNT,
        &DEV_GENESIS_KEY.private_key(),
        milliseconds_since_epoch(),
        0,
        vec![*DEV_GENESIS_HASH],
    ));
    assert_eq!(
        Amount::zero(),
        node.online_reps.lock().unwrap().online_weight()
    );

    // Without rep crawler
    let channel = make_fake_channel(&node);
    node.vote_processor
        .vote_blocking(&vote, &Some(channel.clone()), VoteSource::Live);
    assert_eq!(
        Amount::zero(),
        node.online_reps.lock().unwrap().online_weight()
    );

    // After inserting to rep crawler
    node.rep_crawler
        .force_query(*DEV_GENESIS_HASH, channel.clone());
    node.vote_processor
        .vote_blocking(&vote, &Some(channel.clone()), VoteSource::Live);
    assert_eq!(
        Amount::MAX,
        node.online_reps.lock().unwrap().online_weight()
    );
}

#[test]
fn online_reps_election() {
    let mut system = System::new();
    let mut flags = NodeFlags::default();
    flags.disable_rep_crawler = true;
    let node = system.build_node().flags(flags).finish();

    // Start election
    let key = KeyPair::new();
    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(1000),
        key.public_key().into(),
        &DEV_GENESIS_KEY,
        node.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));

    node.process_active(send1.clone());
    assert_timely_eq(Duration::from_secs(5), || node.active.len(), 1);

    // Process vote for ongoing election
    let vote = Arc::new(Vote::new(
        *DEV_GENESIS_ACCOUNT,
        &DEV_GENESIS_KEY.private_key(),
        milliseconds_since_epoch(),
        0,
        vec![send1.hash()],
    ));
    assert_eq!(
        Amount::zero(),
        node.online_reps.lock().unwrap().online_weight()
    );

    let channel = make_fake_channel(&node);
    node.vote_processor
        .vote_blocking(&vote, &Some(channel.clone()), VoteSource::Live);

    assert_eq!(
        Amount::MAX - Amount::nano(1000),
        node.online_reps.lock().unwrap().online_weight()
    );
}

#[test]
fn vote_republish() {
    let mut system = System::new();
    let node1 = system.make_node();
    let node2 = system.make_node();
    let key2 = KeyPair::new();
    // by not setting a private key on node1's wallet for genesis account, it is stopped from voting
    let wallet_id = node2.wallets.wallet_ids()[0];
    node2
        .wallets
        .insert_adhoc2(&wallet_id, &key2.private_key(), true)
        .unwrap();

    // send1 and send2 are forks of each other
    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(1000),
        key2.public_key().into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));
    let send2 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(2000),
        key2.public_key().into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));

    // process send1 first, this will make sure send1 goes into the ledger and an election is started
    node1.process_active(send1.clone());
    assert_timely(
        Duration::from_secs(5),
        || node2.block_exists(&send1.hash()),
        "block not found on node2",
    );
    assert_timely(
        Duration::from_secs(5),
        || node1.active.active(&send1),
        "not active on node 1",
    );
    assert_timely(
        Duration::from_secs(5),
        || node2.active.active(&send1),
        "not active on node 2",
    );

    // now process send2, send2 will not go in the ledger because only the first block of a fork goes in the ledger
    node1.process_active(send2.clone());
    assert_timely(
        Duration::from_secs(5),
        || node1.active.active(&send2),
        "send2 not active on node 2",
    );

    // send2 cannot be synced because it is not in the ledger of node1, it is only in the election object in RAM on node1
    assert_eq!(node1.block_exists(&send2.hash()), false);

    // the vote causes the election to reach quorum and for the vote (and block?) to be published from node1 to node2
    let vote = Arc::new(Vote::new_final(&DEV_GENESIS_KEY, vec![send2.hash()]));
    let channel = make_fake_channel(&node1);
    node1
        .vote_processor_queue
        .vote(vote, &channel, VoteSource::Live);

    // FIXME: there is a race condition here, if the vote arrives before the block then the vote is wasted and the test fails
    // we could resend the vote but then there is a race condition between the vote resending and the election reaching quorum on node1
    // the proper fix would be to observe on node2 that both the block and the vote arrived in whatever order
    // the real node will do a confirm request if it needs to find a lost vote

    // check that send2 won on both nodes
    assert_timely(
        Duration::from_secs(5),
        || node1.blocks_confirmed(&[send2.clone()]),
        "not confirmed on node1",
    );
    assert_timely(
        Duration::from_secs(5),
        || node2.blocks_confirmed(&[send2.clone()]),
        "not confirmed on node2",
    );

    // check that send1 is deleted from the ledger on nodes
    assert_eq!(node1.block_exists(&send1.hash()), false);
    assert_eq!(node2.block_exists(&send1.hash()), false);
    assert_timely_eq(
        Duration::from_secs(5),
        || node1.balance(&key2.public_key()),
        Amount::nano(2000),
    );
    assert_timely_eq(
        Duration::from_secs(5),
        || node2.balance(&key2.public_key()),
        Amount::nano(2000),
    );
}

// This test places block send1 onto every node. Then it creates block send2 (which is a fork of send1) and sends it to node1.
// Then it sends a vote for send2 to node1 and expects node2 to also get the block plus vote and confirm send2.
// TODO: This test enforces the order block followed by vote on node1, should vote followed by block also work? It doesn't currently.
#[test]
fn vote_by_hash_republish() {
    let mut system = System::new();
    let node1 = system.make_node();
    let node2 = system.make_node();
    let key2 = KeyPair::new();
    // by not setting a private key on node1's wallet for genesis account, it is stopped from voting
    let wallet_id = node2.wallets.wallet_ids()[0];
    node2
        .wallets
        .insert_adhoc2(&wallet_id, &key2.private_key(), true)
        .unwrap();

    // send1 and send2 are forks of each other
    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(1000),
        key2.public_key().into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));
    let send2 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(2000),
        key2.public_key().into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));

    // give block send1 to node1 and check that an election for send1 starts on both nodes
    node1.process_active(send1.clone());
    assert_timely(
        Duration::from_secs(5),
        || node1.active.active(&send1),
        "not active on node 1",
    );
    assert_timely(
        Duration::from_secs(5),
        || node2.active.active(&send1),
        "not active on node 2",
    );

    // give block send2 to node1 and wait until the block is received and processed by node1
    node1.network.publish_filter.clear_all();
    node1.process_active(send2.clone());
    assert_timely(
        Duration::from_secs(5),
        || node1.active.active(&send2),
        "send2 not active on node 1",
    );

    // construct a vote for send2 in order to overturn send1
    let vote = Arc::new(Vote::new_final(&DEV_GENESIS_KEY, vec![send2.hash()]));
    let channel = make_fake_channel(&node1);
    node1
        .vote_processor_queue
        .vote(vote, &channel, VoteSource::Live);

    // send2 should win on both nodes
    assert_timely(
        Duration::from_secs(5),
        || node1.blocks_confirmed(&[send2.clone()]),
        "not confirmed on node1",
    );
    assert_timely(
        Duration::from_secs(5),
        || node2.blocks_confirmed(&[send2.clone()]),
        "not confirmed on node2",
    );
    assert_eq!(node1.block_exists(&send1.hash()), false);
    assert_eq!(node2.block_exists(&send1.hash()), false);
}

#[test]
fn fork_election_invalid_block_signature() {
    let mut system = System::new();
    let node1 = system.make_node();

    // send1 and send2 are forks of each other
    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(1000),
        (*DEV_GENESIS_ACCOUNT).into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));
    let send2 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(2000),
        (*DEV_GENESIS_ACCOUNT).into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));
    let mut send3 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::nano(2000),
        (*DEV_GENESIS_ACCOUNT).into(),
        &DEV_GENESIS_KEY,
        node1.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));
    send3.set_block_signature(&Signature::new()); // Invalid signature

    let channel = make_fake_channel(&node1);
    node1.inbound_message_queue.put(
        DeserializedMessage::new(
            Message::Publish(Publish::new_forward(send1.clone())),
            node1.network_params.network.protocol_info(),
        ),
        channel.clone(),
    );
    assert_timely(
        Duration::from_secs(5),
        || node1.active.active(&send1),
        "not active on node 1",
    );
    let election = node1.active.election(&send1.qualified_root()).unwrap();
    assert_eq!(1, election.mutex.lock().unwrap().last_blocks.len());

    node1.inbound_message_queue.put(
        DeserializedMessage::new(
            Message::Publish(Publish::new_forward(send3)),
            node1.network_params.network.protocol_info(),
        ),
        channel.clone(),
    );
    node1.inbound_message_queue.put(
        DeserializedMessage::new(
            Message::Publish(Publish::new_forward(send2.clone())),
            node1.network_params.network.protocol_info(),
        ),
        channel.clone(),
    );
    assert_timely(
        Duration::from_secs(3),
        || election.mutex.lock().unwrap().last_blocks.len() > 1,
        "block len was < 2",
    );
    assert_eq!(
        election
            .mutex
            .lock()
            .unwrap()
            .last_blocks
            .get(&send2.hash())
            .unwrap()
            .block_signature(),
        send2.block_signature()
    );
}

#[test]
fn confirm_back() {
    let mut system = System::new();
    let node = system.make_node();
    let key = KeyPair::new();

    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::raw(1),
        key.public_key().into(),
        &DEV_GENESIS_KEY,
        node.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));
    let open = BlockEnum::State(StateBlock::new(
        key.public_key(),
        BlockHash::zero(),
        key.public_key(),
        Amount::raw(1),
        send1.hash().into(),
        &key,
        node.work_generate_dev(key.public_key().into()),
    ));
    let send2 = BlockEnum::State(StateBlock::new(
        key.public_key(),
        open.hash(),
        key.public_key(),
        Amount::zero(),
        (*DEV_GENESIS_ACCOUNT).into(),
        &key,
        node.work_generate_dev(open.hash().into()),
    ));

    node.process_active(send1.clone());
    node.process_active(open.clone());
    node.process_active(send2.clone());

    assert_timely(
        Duration::from_secs(5),
        || node.block_exists(&send2.hash()),
        "send2 not found",
    );

    start_election(&node, &send1.hash());
    start_election(&node, &open.hash());
    start_election(&node, &send2.hash());
    assert_eq!(node.active.len(), 3);
    let vote = Arc::new(Vote::new_final(&DEV_GENESIS_KEY, vec![send2.hash()]));
    let channel = make_fake_channel(&node);
    node.vote_processor_queue
        .vote(vote, &channel, VoteSource::Live);
    assert_timely_eq(Duration::from_secs(10), || node.active.len(), 0);
}

#[test]
fn rollback_vote_self() {
    let mut system = System::new();
    let mut flags = NodeFlags::default();
    flags.disable_request_loop = true;
    let node = system.build_node().flags(flags).finish();
    let wallet_id = node.wallets.wallet_ids()[0];
    let key = KeyPair::new();

    // send half the voting weight to a non voting rep to ensure quorum cannot be reached
    let send1 = BlockEnum::State(StateBlock::new(
        *DEV_GENESIS_ACCOUNT,
        *DEV_GENESIS_HASH,
        *DEV_GENESIS_ACCOUNT,
        Amount::MAX - Amount::MAX / 2,
        key.public_key().into(),
        &DEV_GENESIS_KEY,
        node.work_generate_dev((*DEV_GENESIS_HASH).into()),
    ));

    let open = BlockEnum::State(StateBlock::new(
        key.public_key(),
        BlockHash::zero(),
        key.public_key(),
        Amount::MAX / 2,
        send1.hash().into(),
        &key,
        node.work_generate_dev(key.public_key().into()),
    ));

    // send 1 raw
    let send2 = BlockEnum::State(StateBlock::new(
        key.public_key(),
        open.hash(),
        key.public_key(),
        open.balance() - Amount::raw(1),
        (*DEV_GENESIS_ACCOUNT).into(),
        &key,
        node.work_generate_dev(open.hash().into()),
    ));

    // fork of send2 block
    let fork = BlockEnum::State(StateBlock::new(
        key.public_key(),
        open.hash(),
        key.public_key(),
        open.balance() - Amount::raw(2),
        (*DEV_GENESIS_ACCOUNT).into(),
        &key,
        node.work_generate_dev(open.hash().into()),
    ));

    // Process and mark the first 2 blocks as confirmed to allow voting
    node.process(send1.clone()).unwrap();
    node.process(open.clone()).unwrap();
    node.confirm(open.hash());

    // wait until the rep weights have caught up with the weight transfer
    assert_timely_eq(
        Duration::from_secs(5),
        || node.ledger.weight(&key.public_key()),
        Amount::MAX / 2,
    );

    // process forked blocks, send2 will be the winner because it was first and there are no votes yet
    node.process_active(send2.clone());
    assert_timely(
        Duration::from_secs(5),
        || node.active.election(&send2.qualified_root()).is_some(),
        "election not found",
    );
    let election = node.active.election(&send2.qualified_root()).unwrap();
    node.process_active(fork.clone());
    assert_timely_eq(
        Duration::from_secs(5),
        || election.mutex.lock().unwrap().last_blocks.len(),
        2,
    );
    assert_eq!(
        election
            .mutex
            .lock()
            .unwrap()
            .status
            .winner
            .as_ref()
            .unwrap()
            .hash(),
        send2.hash()
    );

    {
        // The write guard prevents the block processor from performing the rollback
        let _write_guard = node.ledger.write_queue.wait(Writer::Testing);

        assert_eq!(0, node.active.votes_with_weight(&election).len());
        // Vote with key to switch the winner
        node.active.vote_applier.vote(
            &election,
            &key.public_key(),
            0,
            &fork.hash(),
            VoteSource::Live,
        );
        assert_eq!(1, node.active.votes_with_weight(&election).len());
        // The winner changed
        assert_eq!(
            election
                .mutex
                .lock()
                .unwrap()
                .status
                .winner
                .as_ref()
                .unwrap()
                .hash(),
            fork.hash(),
        );

        // Insert genesis key in the wallet
        node.wallets
            .insert_adhoc2(&wallet_id, &DEV_GENESIS_KEY.private_key(), true)
            .unwrap();

        // Without the rollback being finished, the aggregator should not reply with any vote
        let channel = make_fake_channel(&node);

        node.request_aggregator
            .request(vec![(send2.hash(), send2.root())], channel);

        assert_always_eq(
            Duration::from_secs(1),
            || {
                node.stats.count(
                    StatType::RequestAggregatorReplies,
                    DetailType::NormalVote,
                    Direction::Out,
                )
            },
            0,
        );

        // Going out of the scope allows the rollback to complete
    }

    // A vote is eventually generated from the local representative
    let is_genesis_vote = |info: &&VoteWithWeightInfo| info.representative == *DEV_GENESIS_ACCOUNT;

    assert_timely_eq(
        Duration::from_secs(5),
        || node.active.votes_with_weight(&election).len(),
        2,
    );
    let votes_with_weight = node.active.votes_with_weight(&election);
    assert_eq!(1, votes_with_weight.iter().filter(is_genesis_vote).count());
    let vote = votes_with_weight.iter().find(is_genesis_vote).unwrap();
    assert_eq!(fork.hash(), vote.hash);
}
