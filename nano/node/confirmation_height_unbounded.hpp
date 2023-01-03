#pragma once

#include <nano/lib/numbers.hpp>
#include <nano/lib/rsnano.hpp>
#include <nano/lib/threading.hpp>
#include <nano/lib/timer.hpp>
#include <nano/secure/store.hpp>

#include <chrono>
#include <unordered_map>

namespace nano
{
class ledger;
class read_transaction;
class logging;
class logger_mt;
class write_database_queue;
class write_guard;

class confirmation_height_unbounded final
{
public:
	confirmation_height_unbounded (nano::ledger &, nano::stat &, nano::write_database_queue &, std::chrono::milliseconds batch_separate_pending_min_time, nano::logging const &, nano::logger_mt &, uint64_t & batch_write_size, std::function<void (std::vector<std::shared_ptr<nano::block>> const &)> const & cemented_callback, std::function<void (nano::block_hash const &)> const & already_cemented_callback, std::function<uint64_t ()> const & awaiting_processing_size_query);
	confirmation_height_unbounded (confirmation_height_unbounded const &) = delete;
	confirmation_height_unbounded (confirmation_height_unbounded &&) = delete;
	~confirmation_height_unbounded ();
	bool pending_empty () const;
	void clear_process_vars ();
	void process (std::shared_ptr<nano::block> original_block);
	void cement_blocks (nano::write_guard &);
	bool has_iterated_over_block (nano::block_hash const &) const;
	void stop ();

	rsnano::ConfirmationHeightUnboundedHandle * handle;

private:
	class conf_height_details_shared_ptr
	{
	public:
		conf_height_details_shared_ptr () :
			handle{ nullptr }
		{
		}
		conf_height_details_shared_ptr (rsnano::ConfHeightDetailsSharedPtrHandle * handle_a) :
			handle{ handle_a }
		{
		}
		conf_height_details_shared_ptr (conf_height_details_shared_ptr const & other_a)
		{
			if (other_a.handle == nullptr)
			{
				handle = nullptr;
			}
			else
			{
				handle = rsnano::rsn_conf_height_details_shared_ptr_clone (other_a.handle);
			}
		}
		conf_height_details_shared_ptr (conf_height_details_shared_ptr &&) = delete;
		~conf_height_details_shared_ptr ()
		{
			if (handle != nullptr)
			{
				rsnano::rsn_conf_height_details_shared_ptr_destroy (handle);
			}
		}
		conf_height_details_shared_ptr & operator= (conf_height_details_shared_ptr const & other_a)
		{
			if (handle != nullptr)
			{
				rsnano::rsn_conf_height_details_shared_ptr_destroy (handle);
			}
			if (other_a.handle == nullptr)
			{
				handle = nullptr;
			}
			else
			{
				handle = rsnano::rsn_conf_height_details_shared_ptr_clone (other_a.handle);
			}
			return *this;
		}

		std::vector<nano::block_hash> get_source_block_callback_data () const
		{
			std::vector<nano::block_hash> result;
			rsnano::U256ArrayDto dto;
			rsnano::rsn_conf_height_details_shared_source_block_callback_data (handle, &dto);
			for (int i = 0; i < dto.count; ++i)
			{
				nano::block_hash hash;
				std::copy (std::begin (dto.items[i]), std::end (dto.items[i]), std::begin (hash.bytes));
				result.push_back (hash);
			}
			rsnano::rsn_u256_array_destroy (&dto);

			return result;
		}
		void set_source_block_callback_data (std::vector<nano::block_hash> const & data_a)
		{
			std::vector<uint8_t const *> tmp;
			for (const auto & i : data_a)
			{
				tmp.push_back (i.bytes.data ());
			}
			rsnano::rsn_conf_height_details_shared_set_source_block_callback_data (handle, tmp.data (), tmp.size ());
		}

		uint64_t get_num_blocks_confirmed () const
		{
			return rsnano::rsn_conf_height_details_shared_num_blocks_confirmed (handle);
		}

		void set_num_blocks_confirmed (uint64_t num)
		{
			rsnano::rsn_conf_height_details_shared_set_num_blocks_confirmed (handle, num);
		}

		std::vector<nano::block_hash> get_block_callback_data () const
		{
			std::vector<nano::block_hash> result;
			rsnano::U256ArrayDto dto;
			rsnano::rsn_conf_height_details_shared_block_callback_data (handle, &dto);
			for (int i = 0; i < dto.count; ++i)
			{
				nano::block_hash hash;
				std::copy (std::begin (dto.items[i]), std::end (dto.items[i]), std::begin (hash.bytes));
				result.push_back (hash);
			}
			rsnano::rsn_u256_array_destroy (&dto);

			return result;
		}

		void add_block_callback_data (nano::block_hash const & hash)
		{
			rsnano::rsn_conf_height_details_shared_add_block_callback_data (handle, hash.bytes.data ());
		}

		void set_block_callback_data (std::vector<nano::block_hash> const & data_a)
		{
			std::vector<uint8_t const *> tmp;
			for (const auto & i : data_a)
			{
				tmp.push_back (i.bytes.data ());
			}
			rsnano::rsn_conf_height_details_shared_set_block_callback_data (handle, tmp.data (), tmp.size ());
		}

		uint64_t get_height () const
		{
			return rsnano::rsn_conf_height_details_shared_height (handle);
		}

		nano::account get_account () const
		{
			nano::account account;
			rsnano::rsn_conf_height_details_shared_account (handle, account.bytes.data ());
			return account;
		}

		bool is_null ()
		{
			return handle == nullptr;
		}

		void destroy ()
		{
			if (handle != nullptr)
			{
				rsnano::rsn_conf_height_details_shared_ptr_destroy (handle);
			}
			handle = nullptr;
		}
		rsnano::ConfHeightDetailsSharedPtrHandle * handle;
	};

	class conf_height_details_weak_ptr
	{
	public:
		conf_height_details_weak_ptr () :
			handle{ nullptr }
		{
		}
		conf_height_details_weak_ptr (rsnano::ConfHeightDetailsWeakPtrHandle * handle_a) :
			handle{ handle_a }
		{
		}
		conf_height_details_weak_ptr (conf_height_details_weak_ptr const & other_a) :
			handle{ rsnano::rsn_conf_height_details_weak_ptr_clone (other_a.handle) }
		{
		}
		conf_height_details_weak_ptr (conf_height_details_shared_ptr const & ptr) :
			handle{ rsnano::rsn_conf_height_details_shared_ptr_to_weak (ptr.handle) }
		{
		}
		conf_height_details_weak_ptr (conf_height_details_weak_ptr &&) = delete;
		~conf_height_details_weak_ptr ()
		{
			if (handle != nullptr)
			{
				rsnano::rsn_conf_height_details_weak_ptr_destroy (handle);
			}
		}
		conf_height_details_weak_ptr & operator= (conf_height_details_weak_ptr const & other_a)
		{
			if (handle != nullptr)
			{
				rsnano::rsn_conf_height_details_weak_ptr_destroy (handle);
			}
			handle = rsnano::rsn_conf_height_details_weak_ptr_clone (other_a.handle);
			return *this;
		}
		bool expired ()
		{
			return rsnano::rsn_conf_height_details_weak_expired (handle);
		}
		conf_height_details_shared_ptr upgrade ()
		{
			return conf_height_details_shared_ptr{ rsnano::rsn_conf_height_details_weak_upgrade (handle) };
		}
		rsnano::ConfHeightDetailsWeakPtrHandle * handle;
	};

	class conf_height_details final
	{
	public:
		conf_height_details (nano::account const &, nano::block_hash const &, uint64_t, uint64_t, std::vector<nano::block_hash> const &);
		conf_height_details (rsnano::ConfHeightDetailsHandle * handle_a) :
			handle{ handle_a }
		{
		}
		conf_height_details (conf_height_details const &);
		conf_height_details (conf_height_details &&) = delete;
		~conf_height_details ();
		conf_height_details & operator= (conf_height_details const &);
		rsnano::ConfHeightDetailsHandle * handle;
		nano::account get_account () const;
		nano::block_hash get_hash () const;
		uint64_t get_height () const;
		uint64_t get_num_blocks_confirmed () const;
		std::vector<nano::block_hash> get_block_callback_data () const;
		void set_block_callback_data (std::vector<nano::block_hash> const &);
		void add_block_callback_data (nano::block_hash const & hash);
	};

	class receive_source_pair final
	{
	public:
		receive_source_pair (conf_height_details_shared_ptr const &, nano::block_hash const &);

		conf_height_details_shared_ptr receive_details;
		nano::block_hash source_hash;
	};

	class preparation_data final
	{
	public:
		uint64_t block_height;
		uint64_t confirmation_height;
		uint64_t iterated_height;
		rsnano::ConfirmedIteratedPairsIteratorDto account_it;
		nano::account const & account;
		conf_height_details_shared_ptr receive_details;
		bool already_traversed;
		nano::block_hash const & current;
		std::vector<nano::block_hash> const & block_callback_data;
		std::vector<nano::block_hash> const & orig_block_callback_data;
	};

	void collect_unconfirmed_receive_and_sources_for_account (uint64_t, uint64_t, std::shared_ptr<nano::block> const &, nano::block_hash const &, nano::account const &, nano::read_transaction const &, std::vector<receive_source_pair> &, std::vector<nano::block_hash> &, std::vector<nano::block_hash> &, std::shared_ptr<nano::block> original_block);
	void prepare_iterated_blocks_for_cementing (preparation_data &);
	uint64_t block_cache_size () const;
	std::shared_ptr<nano::block> get_block_and_sideband (nano::block_hash const &, nano::transaction const &);

	// Fields:

	// All of the atomic variables here just track the size for use in collect_container_info.
	// This is so that no mutexes are needed during the algorithm itself, which would otherwise be needed
	// for the sake of a rarely used RPC call for debugging purposes. As such the sizes are not being acted
	// upon in any way (does not synchronize with any other data).
	// This allows the load and stores to use relaxed atomic memory ordering.

	nano::timer<std::chrono::milliseconds> timer;

	nano::ledger & ledger;
	nano::stat & stats;
	nano::write_database_queue & write_database_queue;
	std::chrono::milliseconds batch_separate_pending_min_time;
	nano::logger_mt & logger;
	std::atomic<bool> stopped{ false };
	uint64_t & batch_write_size;
	nano::logging const & logging;

	std::function<void (std::vector<std::shared_ptr<nano::block>> const &)> notify_observers_callback;
	std::function<void (nano::block_hash const &)> notify_block_already_cemented_observers_callback;
	std::function<uint64_t ()> awaiting_processing_size_callback;

	friend class confirmation_height_dynamic_algorithm_no_transition_while_pending_Test;
	friend std::unique_ptr<nano::container_info_component> collect_container_info (confirmation_height_unbounded &, std::string const & name_a);
};

std::unique_ptr<nano::container_info_component> collect_container_info (confirmation_height_unbounded &, std::string const & name_a);
}
