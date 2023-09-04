#include "nano/node/transport/traffic_type.hpp"

#include <nano/boost/asio/bind_executor.hpp>
#include <nano/boost/asio/ip/address_v6.hpp>
#include <nano/boost/asio/read.hpp>
#include <nano/lib/rsnanoutils.hpp>
#include <nano/node/node.hpp>
#include <nano/node/transport/socket.hpp>
#include <nano/node/transport/transport.hpp>

#include <boost/format.hpp>

#include <cstdint>
#include <cstdlib>
#include <iterator>
#include <limits>
#include <memory>
#include <utility>

namespace
{
bool is_temporary_error (boost::system::error_code const & ec_a)
{
	switch (ec_a.value ())
	{
#if EAGAIN != EWOULDBLOCK
		case EAGAIN:
#endif

		case EWOULDBLOCK:
		case EINTR:
			return true;
		default:
			return false;
	}
}
}

nano::transport::tcp_socket_facade::tcp_socket_facade (boost::asio::io_context & io_ctx_a) :
	strand{ io_ctx_a.get_executor () },
	tcp_socket{ io_ctx_a },
	io_ctx{ io_ctx_a },
	acceptor{ io_ctx_a }
{
}

nano::transport::tcp_socket_facade::~tcp_socket_facade ()
{
	boost::system::error_code ec;
	close (ec);
}

void nano::transport::tcp_socket_facade::async_connect (boost::asio::ip::tcp::endpoint endpoint_a,
std::function<void (boost::system::error_code const &)> callback_a)
{
	tcp_socket.async_connect (endpoint_a, boost::asio::bind_executor (strand, callback_a));
}

void nano::transport::tcp_socket_facade::async_read (std::shared_ptr<std::vector<uint8_t>> const & buffer_a, size_t len_a, std::function<void (boost::system::error_code const &, std::size_t)> callback_a)
{
	auto this_l{ shared_from_this () };
	boost::asio::post (strand, boost::asio::bind_executor (strand, [buffer_a, callback = std::move (callback_a), len_a, this_l] () mutable {
		boost::asio::async_read (this_l->tcp_socket, boost::asio::buffer (buffer_a->data (), len_a),
		boost::asio::bind_executor (this_l->strand, [buffer_a, callback = std::move (callback), this_l] (boost::system::error_code const & ec, std::size_t len) {
			callback (ec, len);
		}));
	}));
}

void nano::transport::tcp_socket_facade::async_read (std::shared_ptr<nano::transport::buffer_wrapper> const & buffer_a, size_t len_a, std::function<void (boost::system::error_code const &, std::size_t)> callback_a)
{
	auto this_l{ shared_from_this () };
	boost::asio::post (strand, boost::asio::bind_executor (strand, [buffer_a, callback = std::move (callback_a), len_a, this_l] () mutable {
		boost::asio::async_read (this_l->tcp_socket, boost::asio::buffer (buffer_a->data (), len_a),
		boost::asio::bind_executor (this_l->strand, [buffer_a, callback = std::move (callback), this_l] (boost::system::error_code const & ec, std::size_t len) {
			callback (ec, len);
		}));
	}));
}

void nano::transport::tcp_socket_facade::async_write (nano::shared_const_buffer const & buffer_a, std::function<void (boost::system::error_code const &, std::size_t)> callback_a)
{
	nano::async_write (tcp_socket, buffer_a,
	boost::asio::bind_executor (strand,
	[buffer_a, cbk = std::move (callback_a), this_l = shared_from_this ()] (boost::system::error_code ec, std::size_t size) {
		cbk (ec, size);
	}));
}

bool nano::transport::tcp_socket_facade::running_in_this_thread ()
{
	return strand.running_in_this_thread ();
}

void nano::transport::tcp_socket_facade::open (const boost::asio::ip::tcp::endpoint & local, boost::system::error_code & ec_a)
{
	acceptor.open (local.protocol ());
	acceptor.set_option (boost::asio::ip::tcp::acceptor::reuse_address (true));
	acceptor.bind (local, ec_a);
	if (!ec_a)
	{
		acceptor.listen (boost::asio::socket_base::max_listen_connections, ec_a);
	}
}

void nano::transport::tcp_socket_facade::async_accept (
boost::asio::ip::tcp::socket & client_socket,
boost::asio::ip::tcp::endpoint & peer,
std::function<void (boost::system::error_code const &)> callback_a)
{
	acceptor.async_accept (client_socket, peer, boost::asio::bind_executor (strand, callback_a));
}

bool nano::transport::tcp_socket_facade::is_acceptor_open ()
{
	return acceptor.is_open ();
}

void nano::transport::tcp_socket_facade::close_acceptor ()
{
	acceptor.close ();
}

boost::asio::ip::tcp::endpoint nano::transport::tcp_socket_facade::remote_endpoint (boost::system::error_code & ec)
{
	return tcp_socket.remote_endpoint (ec);
}

void nano::transport::tcp_socket_facade::dispatch (std::function<void ()> callback_a)
{
	boost::asio::dispatch (strand, boost::asio::bind_executor (strand, [callback_a, this_l = shared_from_this ()] {
		callback_a ();
	}));
}

void nano::transport::tcp_socket_facade::post (std::function<void ()> callback_a)
{
	boost::asio::post (strand, boost::asio::bind_executor (strand, [callback_a, this_l = shared_from_this ()] {
		callback_a ();
	}));
}

void nano::transport::tcp_socket_facade::close (boost::system::error_code & ec)
{
	if (!closed.exchange (true))
	{
		// Ignore error code for shutdown as it is best-effort
		tcp_socket.shutdown (boost::asio::ip::tcp::socket::shutdown_both, ec);
		tcp_socket.close (ec);
	}
}

nano::transport::buffer_wrapper::buffer_wrapper (std::size_t len) :
	handle{ rsnano::rsn_buffer_create (len) }
{
}

nano::transport::tcp_socket_facade_factory::tcp_socket_facade_factory (boost::asio::io_context & io_ctx_a) :
	io_ctx{ io_ctx_a }
{
}

std::shared_ptr<nano::transport::tcp_socket_facade> nano::transport::tcp_socket_facade_factory::create_socket ()
{
	return std::make_shared<nano::transport::tcp_socket_facade> (io_ctx);
}

nano::transport::buffer_wrapper::buffer_wrapper (rsnano::BufferHandle * handle_a) :
	handle{ handle_a }
{
}

nano::transport::buffer_wrapper::buffer_wrapper (buffer_wrapper && other_a) :
	handle{ other_a.handle }
{
	other_a.handle = nullptr;
}

nano::transport::buffer_wrapper::~buffer_wrapper ()
{
	if (handle)
		rsnano::rsn_buffer_destroy (handle);
}

std::uint8_t * nano::transport::buffer_wrapper::data ()
{
	return rsnano::rsn_buffer_data (handle);
}

std::size_t nano::transport::buffer_wrapper::len () const
{
	return rsnano::rsn_buffer_len (handle);
}

/*
 * socket
 */

nano::transport::socket::socket (rsnano::async_runtime & async_rt_a, endpoint_type_t endpoint_type_a, nano::stats & stats_a,
std::shared_ptr<nano::logger_mt> & logger_a, std::shared_ptr<nano::thread_pool> const & workers_a,
std::chrono::seconds default_timeout_a, std::chrono::seconds silent_connection_tolerance_time_a,
std::chrono::seconds idle_timeout_a,
bool network_timeout_logging_a,
std::shared_ptr<nano::node_observers> observers_a,
std::size_t max_queue_size_a) :
	handle{ rsnano::rsn_socket_create (
	static_cast<uint8_t> (endpoint_type_a),
	new std::shared_ptr<nano::transport::tcp_socket_facade> (std::make_shared<nano::transport::tcp_socket_facade> (async_rt_a.io_ctx)),
	stats_a.handle,
	workers_a->handle,
	default_timeout_a.count (),
	silent_connection_tolerance_time_a.count (),
	idle_timeout_a.count (),
	network_timeout_logging_a,
	nano::to_logger_handle (logger_a),
	new std::weak_ptr<nano::node_observers> (observers_a),
	max_queue_size_a) }
{
}

nano::transport::socket::socket (rsnano::SocketHandle * handle_a) :
	handle{ handle_a }
{
}

nano::transport::socket::~socket ()
{
	rsnano::rsn_socket_destroy (handle);
}

void async_connect_adapter (void * context, rsnano::ErrorCodeDto const * error)
{
	try
	{
		auto ec{ rsnano::dto_to_error_code (*error) };
		auto callback = static_cast<std::function<void (boost::system::error_code const &)> *> (context);
		(*callback) (ec);
	}
	catch (...)
	{
		std::cerr << "exception in async_connect_adapter!" << std::endl;
	}
}

void async_connect_delete_context (void * context)
{
	auto callback = static_cast<std::function<void (boost::system::error_code const &)> *> (context);
	delete callback;
}

boost::asio::ip::tcp::endpoint & nano::transport::socket::get_remote ()
{
	return remote;
}

void nano::transport::socket::start ()
{
	rsnano::rsn_socket_start (handle);
}

void nano::transport::socket::async_connect (nano::tcp_endpoint const & endpoint_a, std::function<void (boost::system::error_code const &)> callback_a)
{
	auto endpoint_dto{ rsnano::endpoint_to_dto (endpoint_a) };
	auto cb_wrapper = new std::function<void (boost::system::error_code const &)> ([callback = std::move (callback_a), this_l = shared_from_this ()] (boost::system::error_code const & ec) {
		callback (ec);
	});
	rsnano::rsn_socket_async_connect (handle, &endpoint_dto, async_connect_adapter, async_connect_delete_context, cb_wrapper);
}

void nano::transport::async_read_adapter (void * context_a, rsnano::ErrorCodeDto const * error_a, std::size_t size_a)
{
	try
	{
		auto ec{ rsnano::dto_to_error_code (*error_a) };
		auto callback = static_cast<std::function<void (boost::system::error_code const &, std::size_t)> *> (context_a);
		(*callback) (ec, size_a);
	}
	catch (...)
	{
		std::cerr << "exception in async_read_adapter!" << std::endl;
	}
}

void nano::transport::async_read_delete_context (void * context_a)
{
	auto callback = static_cast<std::function<void (boost::system::error_code const &, std::size_t)> *> (context_a);
	delete callback;
}

void nano::transport::socket::async_read (std::shared_ptr<std::vector<uint8_t>> const & buffer_a, std::size_t size_a, std::function<void (boost::system::error_code const &, std::size_t)> callback_a)
{
	auto cb_wrapper = new std::function<void (boost::system::error_code const &, std::size_t)> ([callback = std::move (callback_a), this_l = shared_from_this ()] (boost::system::error_code const & ec, std::size_t size) {
		callback (ec, size);
	});
	auto buffer_ptr{ new std::shared_ptr<std::vector<uint8_t>> (buffer_a) };
	rsnano::rsn_socket_async_read (handle, buffer_ptr, size_a, nano::transport::async_read_adapter, nano::transport::async_read_delete_context, cb_wrapper);
}

void nano::transport::socket::async_read (std::shared_ptr<nano::transport::buffer_wrapper> const & buffer_a, std::size_t size_a, std::function<void (boost::system::error_code const &, std::size_t)> callback_a)
{
	auto cb_wrapper = new std::function<void (boost::system::error_code const &, std::size_t)> ([callback = std::move (callback_a), this_l = shared_from_this ()] (boost::system::error_code const & ec, std::size_t size) {
		callback (ec, size);
	});
	rsnano::rsn_socket_async_read2 (handle, buffer_a->handle, size_a, nano::transport::async_read_adapter, nano::transport::async_read_delete_context, cb_wrapper);
}

void nano::transport::socket::async_write (nano::shared_const_buffer const & buffer_a, std::function<void (boost::system::error_code const &, std::size_t)> callback_a, nano::transport::traffic_type traffic_type)
{
	auto cb_wrapper = new std::function<void (boost::system::error_code const &, std::size_t)> ([callback = std::move (callback_a), this_l = shared_from_this ()] (boost::system::error_code const & ec, std::size_t size) {
		callback (ec, size);
	});

	auto buffer_l = buffer_a.to_bytes ();
	rsnano::rsn_socket_async_write (handle, buffer_l.data (), buffer_l.size (), async_read_adapter, async_read_delete_context, cb_wrapper, static_cast<uint8_t> (traffic_type));
}

/** Set the current timeout of the socket in seconds
 *  timeout occurs when the last socket completion is more than timeout seconds in the past
 *  timeout always applies, the socket always has a timeout
 *  to set infinite timeout, use std::numeric_limits<uint64_t>::max ()
 *  the function checkup() checks for timeout on a regular interval
 */
void nano::transport::socket::set_timeout (std::chrono::seconds timeout_a)
{
	rsnano::rsn_socket_set_timeout (handle, timeout_a.count ());
}

bool nano::transport::socket::has_timed_out () const
{
	return rsnano::rsn_socket_has_timed_out (handle);
}

void nano::transport::socket::set_default_timeout_value (std::chrono::seconds timeout_a)
{
	rsnano::rsn_socket_set_default_timeout_value (handle, timeout_a.count ());
}

std::chrono::seconds nano::transport::socket::get_default_timeout_value () const
{
	return std::chrono::seconds{ rsnano::rsn_socket_default_timeout_value (handle) };
}

void nano::transport::socket::set_silent_connection_tolerance_time (std::chrono::seconds tolerance_time_a)
{
	rsnano::rsn_socket_set_silent_connection_tolerance_time (handle, tolerance_time_a.count ());
}

nano::transport::socket::type_t nano::transport::socket::type () const
{
	return static_cast<nano::transport::socket::type_t> (rsnano::rsn_socket_type (handle));
}

void nano::transport::socket::type_set (nano::transport::socket::type_t type_a)
{
	rsnano::rsn_socket_set_type (handle, static_cast<uint8_t> (type_a));
}

nano::transport::socket::endpoint_type_t nano::transport::socket::endpoint_type () const
{
	return static_cast<nano::transport::socket::endpoint_type_t> (rsnano::rsn_socket_endpoint_type (handle));
}

void nano::transport::socket::close ()
{
	rsnano::rsn_socket_close (handle);
}

void nano::transport::socket::close_internal ()
{
	rsnano::rsn_socket_close_internal (handle);
}

void nano::transport::socket::checkup ()
{
	rsnano::rsn_socket_checkup (handle);
}

bool nano::transport::socket::is_bootstrap_connection ()
{
	return rsnano::rsn_socket_is_bootstrap_connection (handle);
}

bool nano::transport::socket::is_closed ()
{
	return rsnano::rsn_socket_is_closed (handle);
}

bool nano::transport::socket::alive () const
{
	return rsnano::rsn_socket_is_alive (handle);
}

boost::asio::ip::tcp::endpoint nano::transport::socket::remote_endpoint () const
{
	rsnano::EndpointDto result;
	rsnano::rsn_socket_get_remote (handle, &result);
	return rsnano::dto_to_endpoint (result);
}

nano::tcp_endpoint nano::transport::socket::local_endpoint () const
{
	rsnano::EndpointDto dto;
	rsnano::rsn_socket_local_endpoint (handle, &dto);
	return rsnano::dto_to_endpoint (dto);
}

bool nano::transport::socket::max (nano::transport::traffic_type traffic_type) const
{
	return rsnano::rsn_socket_max (handle, static_cast<uint8_t> (traffic_type));
}

bool nano::transport::socket::full (nano::transport::traffic_type traffic_type) const
{
	return rsnano::rsn_socket_full (handle, static_cast<uint8_t> (traffic_type));
}

/*
 * server_socket
 */

nano::transport::server_socket::server_socket (nano::node & node_a, boost::asio::ip::tcp::endpoint local_a, std::size_t max_connections_a) :
	socket_facade{ std::make_shared<nano::transport::tcp_socket_facade> (node_a.io_ctx) },
	socket{ node_a.async_rt, nano::transport::socket::endpoint_type_t::server, *node_a.stats, node_a.logger, node_a.workers,
		std::chrono::seconds::max (),
		node_a.network_params.network.silent_connection_tolerance_time,
		node_a.network_params.network.idle_timeout,
		node_a.config->logging.network_timeout_logging (),
		node_a.observers },
	local{ std::move (local_a) }
{
	auto network_params_dto{ node_a.network_params.to_dto () };
	auto node_config_dto{ node_a.config->to_dto () };
	auto local_dto{ rsnano::endpoint_to_dto (local_a) };
	handle = rsnano::rsn_server_socket_create (
	new std::shared_ptr<nano::transport::tcp_socket_facade> (socket_facade),
	socket.handle,
	node_a.flags.handle,
	&network_params_dto,
	node_a.workers->handle,
	nano::to_logger_handle (node_a.logger),
	new std::shared_ptr<nano::transport::tcp_socket_facade_factory> (std::make_shared<nano::transport::tcp_socket_facade_factory> (node_a.io_ctx)),
	new std::weak_ptr<nano::node_observers> (node_a.observers),
	node_a.stats->handle,
	&node_config_dto,
	max_connections_a,
	&local_dto);
}

nano::transport::server_socket::~server_socket ()
{
	rsnano::rsn_server_socket_destroy (handle);
}

void nano::transport::server_socket::start (boost::system::error_code & ec_a)
{
	rsnano::rsn_server_socket_start (handle);
}

void nano::transport::server_socket::close ()
{
	rsnano::rsn_server_socket_close (handle);
}

boost::asio::ip::network_v6 nano::transport::socket_functions::get_ipv6_subnet_address (boost::asio::ip::address_v6 const & ip_address, std::size_t network_prefix)
{
	return boost::asio::ip::make_network_v6 (ip_address, static_cast<unsigned short> (network_prefix));
}

namespace
{
bool on_connection_callback (void * context, rsnano::SocketHandle * socket_handle, const rsnano::ErrorCodeDto * ec_dto)
{
	auto callback = static_cast<std::function<bool (std::shared_ptr<nano::transport::socket> const &, boost::system::error_code const &)> *> (context);
	auto socket = std::make_shared<nano::transport::socket> (socket_handle);
	auto ec = rsnano::dto_to_error_code (*ec_dto);
	return (*callback) (socket, ec);
}

void delete_on_connection_context (void * handle_a)
{
	auto callback = static_cast<std::function<bool (std::shared_ptr<nano::transport::socket> const &, boost::system::error_code const &)> *> (handle_a);
	delete callback;
}
}

void nano::transport::server_socket::on_connection (std::function<bool (std::shared_ptr<nano::transport::socket> const &, boost::system::error_code const &)> callback_a)
{
	auto context = new std::function<bool (std::shared_ptr<nano::transport::socket> const &, boost::system::error_code const &)> (callback_a);
	rsnano::rsn_server_socket_on_connection (handle, on_connection_callback, context, delete_on_connection_context);
	return;
}

std::shared_ptr<nano::transport::socket> nano::transport::create_client_socket (nano::node & node_a, std::size_t write_queue_size)
{
	return std::make_shared<nano::transport::socket> (node_a.async_rt, nano::transport::socket::endpoint_type_t::client, *node_a.stats, node_a.logger, node_a.workers,
	node_a.config->tcp_io_timeout,
	node_a.network_params.network.silent_connection_tolerance_time,
	node_a.network_params.network.idle_timeout,
	node_a.config->logging.network_timeout_logging (),
	node_a.observers,
	write_queue_size);
}

nano::transport::weak_socket_wrapper::weak_socket_wrapper (rsnano::SocketWeakHandle * handle_a) :
	handle{ handle_a }
{
}

nano::transport::weak_socket_wrapper::weak_socket_wrapper (std::shared_ptr<nano::transport::socket> & socket) :
	handle{ rsnano::rsn_socket_to_weak_handle (socket->handle) }
{
}

nano::transport::weak_socket_wrapper::~weak_socket_wrapper ()
{
	rsnano::rsn_weak_socket_destroy (handle);
}

std::shared_ptr<nano::transport::socket> nano::transport::weak_socket_wrapper::lock ()
{
	auto socket_handle = rsnano::rsn_weak_socket_to_socket (handle);
	std::shared_ptr<nano::transport::socket> socket;
	if (socket_handle)
	{
		socket = std::make_shared<nano::transport::socket> (socket_handle);
	}
	return socket;
}

bool nano::transport::weak_socket_wrapper::expired () const
{
	return rsnano::rsn_weak_socket_expired (handle);
}

std::string nano::transport::socket_type_to_string (nano::transport::socket::type_t type)
{
	rsnano::StringDto dto;
	rsnano::rsn_socket_type_to_string (static_cast<uint8_t> (type), &dto);
	return rsnano::convert_dto_to_string (dto);
}
