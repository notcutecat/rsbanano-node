#include <nano/node/lmdb/frontier_store.hpp>
#include <nano/node/lmdb/lmdb.hpp>
#include <nano/secure/parallel_traversal.hpp>

nano::lmdb::frontier_store::frontier_store (nano::lmdb::store & store) :
	store{ store }
{
	handle = rsnano::rsn_lmdb_frontier_store_create (store.env ().handle);
}

nano::lmdb::frontier_store::~frontier_store ()
{
	rsnano::rsn_lmdb_frontier_store_destroy (handle);
}

void nano::lmdb::frontier_store::put (nano::write_transaction const & transaction, nano::block_hash const & hash, nano::account const & account)
{
	rsnano::rsn_lmdb_frontier_store_put (handle, transaction.get_rust_handle (), hash.bytes.data (), account.bytes.data ());
}

nano::account nano::lmdb::frontier_store::get (nano::transaction const & transaction, nano::block_hash const & hash) const
{
	nano::account result;
	rsnano::rsn_lmdb_frontier_store_get (handle, transaction.get_rust_handle (), hash.bytes.data (), result.bytes.data ());
	return result;
}

void nano::lmdb::frontier_store::del (nano::write_transaction const & transaction, nano::block_hash const & hash)
{
	auto status = store.del (transaction, tables::frontiers, hash);
	store.release_assert_success (status);
}

nano::store_iterator<nano::block_hash, nano::account> nano::lmdb::frontier_store::begin (nano::transaction const & transaction) const
{
	return store.make_iterator<nano::block_hash, nano::account> (transaction, tables::frontiers);
}

nano::store_iterator<nano::block_hash, nano::account> nano::lmdb::frontier_store::begin (nano::transaction const & transaction, nano::block_hash const & hash) const
{
	return store.make_iterator<nano::block_hash, nano::account> (transaction, tables::frontiers, nano::db_val<MDB_val> (hash));
}

nano::store_iterator<nano::block_hash, nano::account> nano::lmdb::frontier_store::end () const
{
	return nano::store_iterator<nano::block_hash, nano::account> (nullptr);
}

void nano::lmdb::frontier_store::for_each_par (std::function<void (nano::read_transaction const &, nano::store_iterator<nano::block_hash, nano::account>, nano::store_iterator<nano::block_hash, nano::account>)> const & action_a) const
{
	parallel_traversal<nano::uint256_t> (
	[&action_a, this] (nano::uint256_t const & start, nano::uint256_t const & end, bool const is_last) {
		auto transaction (this->store.tx_begin_read ());
		action_a (*transaction, this->begin (*transaction, start), !is_last ? this->begin (*transaction, end) : this->end ());
	});
}

MDB_dbi nano::lmdb::frontier_store::table_handle () const
{
	return rsnano::rsn_lmdb_frontier_store_table_handle (handle);
}

void nano::lmdb::frontier_store::set_table_handle (MDB_dbi dbi)
{
	rsnano::rsn_lmdb_frontier_store_set_table_handle (handle, dbi);
}
