#include <nano/node/common.hpp>
#include <nano/node/network.hpp>
#include <nano/secure/buffer.hpp>

#include <gtest/gtest.h>

#include <boost/algorithm/string.hpp>
#include <boost/variant/get.hpp>

TEST (message, keepalive_serialization)
{
	nano::keepalive request1{ nano::dev::network_params.network };
	std::vector<uint8_t> bytes;
	{
		nano::vectorstream stream (bytes);
		request1.serialize (stream);
	}
	auto error (false);
	nano::bufferstream stream (bytes.data (), bytes.size ());
	nano::message_header header (error, stream);
	ASSERT_FALSE (error);
	nano::keepalive request2 (error, stream, header);
	ASSERT_FALSE (error);
	ASSERT_EQ (request1, request2);
}

TEST (message, keepalive_deserialize)
{
	nano::keepalive message1{ nano::dev::network_params.network };
	auto peers{ message1.get_peers () };
	peers[0] = nano::endpoint (boost::asio::ip::address_v6::loopback (), 10000);
	message1.set_peers (peers);
	std::vector<uint8_t> bytes;
	{
		nano::vectorstream stream (bytes);
		message1.serialize (stream);
	}
	nano::bufferstream stream (bytes.data (), bytes.size ());
	auto error (false);
	nano::message_header header (error, stream);
	ASSERT_FALSE (error);
	ASSERT_EQ (nano::message_type::keepalive, header.get_type ());
	nano::keepalive message2 (error, stream, header);
	ASSERT_FALSE (error);
	ASSERT_EQ (message1.get_peers (), message2.get_peers ());
}

TEST (message, publish_serialization)
{
	nano::keypair key1;
	nano::publish publish{ nano::dev::network_params.network, std::make_shared<nano::send_block> (0, 1, 2, key1.prv, key1.pub, 5) };
	ASSERT_EQ (nano::block_type::send, publish.get_header ().block_type ());
	std::vector<uint8_t> bytes;
	{
		nano::vectorstream stream (bytes);
		publish.get_header ().serialize (stream);
	}
	ASSERT_EQ (8, bytes.size ());
	ASSERT_EQ (0x52, bytes[0]);
	ASSERT_EQ (0x41, bytes[1]);
	ASSERT_EQ (nano::dev::network_params.network.protocol_version, bytes[2]);
	ASSERT_EQ (nano::dev::network_params.network.protocol_version, bytes[3]);
	ASSERT_EQ (nano::dev::network_params.network.protocol_version_min, bytes[4]);
	ASSERT_EQ (static_cast<uint8_t> (nano::message_type::publish), bytes[5]);
	ASSERT_EQ (0x00, bytes[6]); // extensions
	ASSERT_EQ (static_cast<uint8_t> (nano::block_type::send), bytes[7]);
	nano::bufferstream stream (bytes.data (), bytes.size ());
	auto error (false);
	nano::message_header header (error, stream);
	ASSERT_FALSE (error);
	ASSERT_EQ (nano::dev::network_params.network.protocol_version_min, header.get_version_min ());
	ASSERT_EQ (nano::dev::network_params.network.protocol_version, header.get_version_using ());
	ASSERT_EQ (nano::dev::network_params.network.protocol_version, header.get_version_max ());
	ASSERT_EQ (nano::message_type::publish, header.get_type ());
}

TEST (message, confirm_ack_hash_serialization)
{
	std::vector<nano::block_hash> hashes;
	for (auto i (hashes.size ()); i < nano::network::confirm_ack_hashes_max; i++)
	{
		nano::keypair key1;
		nano::block_hash previous;
		nano::random_pool::generate_block (previous.bytes.data (), previous.bytes.size ());
		nano::state_block block (key1.pub, previous, key1.pub, 2, 4, key1.prv, key1.pub, 5);
		hashes.push_back (block.hash ());
	}
	nano::keypair representative1;
	auto vote (std::make_shared<nano::vote> (representative1.pub, representative1.prv, 0, 0, hashes));
	nano::confirm_ack con1{ nano::dev::network_params.network, vote };
	std::vector<uint8_t> bytes;
	{
		nano::vectorstream stream1 (bytes);
		con1.serialize (stream1);
	}
	nano::bufferstream stream2 (bytes.data (), bytes.size ());
	bool error (false);
	nano::message_header header (error, stream2);
	nano::confirm_ack con2 (error, stream2, header);
	ASSERT_FALSE (error);
	ASSERT_EQ (con1, con2);
	ASSERT_EQ (hashes, con2.get_vote ()->hashes ());
	// Check overflow with max hashes
	ASSERT_EQ (header.count_get (), hashes.size ());
	ASSERT_EQ (header.block_type (), nano::block_type::not_a_block);
}

TEST (message, confirm_req_serialization)
{
	nano::keypair key1;
	nano::keypair key2;
	auto block (std::make_shared<nano::send_block> (0, key2.pub, 200, nano::keypair ().prv, 2, 3));
	nano::confirm_req req{ nano::dev::network_params.network, block };
	std::vector<uint8_t> bytes;
	{
		nano::vectorstream stream (bytes);
		req.serialize (stream);
	}
	auto error (false);
	nano::bufferstream stream2 (bytes.data (), bytes.size ());
	nano::message_header header (error, stream2);
	nano::confirm_req req2 (error, stream2, header);
	ASSERT_FALSE (error);
	ASSERT_EQ (req, req2);
	ASSERT_EQ (*req.get_block (), *req2.get_block ());
}

TEST (message, confirm_req_hash_serialization)
{
	nano::keypair key1;
	nano::keypair key2;
	nano::send_block block (1, key2.pub, 200, nano::keypair ().prv, 2, 3);
	nano::confirm_req req{ nano::dev::network_params.network, block.hash (), block.root () };
	std::vector<uint8_t> bytes;
	{
		nano::vectorstream stream (bytes);
		req.serialize (stream);
	}
	auto error (false);
	nano::bufferstream stream2 (bytes.data (), bytes.size ());
	nano::message_header header (error, stream2);
	nano::confirm_req req2 (error, stream2, header);
	ASSERT_FALSE (error);
	ASSERT_EQ (req, req2);
	ASSERT_EQ (req.get_roots_hashes (), req2.get_roots_hashes ());
	ASSERT_EQ (header.block_type (), nano::block_type::not_a_block);
	ASSERT_EQ (header.count_get (), req.get_roots_hashes ().size ());
}

TEST (message, confirm_req_hash_batch_serialization)
{
	nano::keypair key;
	nano::keypair representative;
	std::vector<std::pair<nano::block_hash, nano::root>> roots_hashes;
	nano::state_block open (key.pub, 0, representative.pub, 2, 4, key.prv, key.pub, 5);
	roots_hashes.push_back (std::make_pair (open.hash (), open.root ()));
	for (auto i (roots_hashes.size ()); i < 7; i++)
	{
		nano::keypair key1;
		nano::block_hash previous;
		nano::random_pool::generate_block (previous.bytes.data (), previous.bytes.size ());
		nano::state_block block (key1.pub, previous, representative.pub, 2, 4, key1.prv, key1.pub, 5);
		roots_hashes.push_back (std::make_pair (block.hash (), block.root ()));
	}
	roots_hashes.push_back (std::make_pair (open.hash (), open.root ()));
	nano::confirm_req req{ nano::dev::network_params.network, roots_hashes };
	std::vector<uint8_t> bytes;
	{
		nano::vectorstream stream (bytes);
		req.serialize (stream);
	}
	auto error (false);
	nano::bufferstream stream2 (bytes.data (), bytes.size ());
	nano::message_header header (error, stream2);
	nano::confirm_req req2 (error, stream2, header);
	ASSERT_FALSE (error);
	ASSERT_EQ (req, req2);
	ASSERT_EQ (req.get_roots_hashes (), req2.get_roots_hashes ());
	ASSERT_EQ (req.get_roots_hashes (), roots_hashes);
	ASSERT_EQ (req2.get_roots_hashes (), roots_hashes);
	ASSERT_EQ (header.block_type (), nano::block_type::not_a_block);
	ASSERT_EQ (header.count_get (), req.get_roots_hashes ().size ());
}

/**
 * Test that a confirm_ack can encode an empty hash set
 */
TEST (confirm_ack, empty_vote_hashes)
{
	nano::keypair key;
	auto vote = std::make_shared<nano::vote> (key.pub, key.prv, 0, 0, std::vector<nano::block_hash>{} /* empty */);
	nano::confirm_ack message{ nano::dev::network_params.network, vote };
}
