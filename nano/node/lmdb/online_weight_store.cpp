#include <nano/node/lmdb/lmdb.hpp>
#include <nano/node/lmdb/online_weight_store.hpp>

namespace
{
nano::store_iterator<uint64_t, nano::amount> to_iterator (rsnano::LmdbIteratorHandle * it_handle)
{
	if (it_handle == nullptr)
	{
		return { nullptr };
	}

	return { std::make_unique<nano::mdb_iterator<uint64_t, nano::amount>> (it_handle) };
}
}

nano::lmdb::online_weight_store::online_weight_store (nano::lmdb::store & store_a) :
	store{ store_a },
	handle{ rsnano::rsn_lmdb_online_weight_store_create (store_a.env ().handle) }
{
}

nano::lmdb::online_weight_store::~online_weight_store ()
{
	rsnano::rsn_lmdb_online_weight_store_destroy (handle);
}

bool nano::lmdb::online_weight_store::open_db (nano::transaction const & txn, uint32_t flags)
{
	return !rsnano::rsn_lmdb_online_weight_store_open_db (handle, txn.get_rust_handle (), flags);
}

void nano::lmdb::online_weight_store::put (nano::write_transaction const & transaction, uint64_t time, nano::amount const & amount)
{
	rsnano::rsn_lmdb_online_weight_store_put (handle, transaction.get_rust_handle (), time, amount.bytes.data ());
}

void nano::lmdb::online_weight_store::del (nano::write_transaction const & transaction, uint64_t time)
{
	rsnano::rsn_lmdb_online_weight_store_del (handle, transaction.get_rust_handle (), time);
}

nano::store_iterator<uint64_t, nano::amount> nano::lmdb::online_weight_store::begin (nano::transaction const & transaction) const
{
	auto it_handle{ rsnano::rsn_lmdb_online_weight_store_begin (handle, transaction.get_rust_handle ()) };
	return to_iterator (it_handle);
}

nano::store_iterator<uint64_t, nano::amount> nano::lmdb::online_weight_store::rbegin (nano::transaction const & transaction) const
{
	auto it_handle{ rsnano::rsn_lmdb_online_weight_store_rbegin (handle, transaction.get_rust_handle ()) };
	return to_iterator (it_handle);
}

nano::store_iterator<uint64_t, nano::amount> nano::lmdb::online_weight_store::end () const
{
	return nano::store_iterator<uint64_t, nano::amount> (nullptr);
}

size_t nano::lmdb::online_weight_store::count (nano::transaction const & transaction) const
{
	return rsnano::rsn_lmdb_online_weight_store_count (handle, transaction.get_rust_handle ());
}

void nano::lmdb::online_weight_store::clear (nano::write_transaction const & transaction)
{
	return rsnano::rsn_lmdb_online_weight_store_clear (handle, transaction.get_rust_handle ());
}

MDB_dbi nano::lmdb::online_weight_store::table_handle () const
{
	return rsnano::rsn_lmdb_online_weight_store_table_handle (handle);
}
