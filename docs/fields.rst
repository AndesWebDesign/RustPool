.. _fields-reference:

================
Field Reference
================

The types references after the field names below are the corresponding Rust type this value will be parsed into. For
convenience, the following should be adhered to:

|
| u8    =>   one byte unsigned integer with range 0 to 255.
| u16   =>   two byte unsigned integer with range 0 to 65535.
| u32   =>   four byte unsigned integer with range 0 to 4294967295.
| u64   =>   eight byte unsigned integer with range 0 to 18446744073709551615.
|
| f32   =>   four byte floating point number with 32 bits of precision.
| f64   =>   eight byte floating point number with 64 bits of precision.
|

**allow_self_select** - bool
    Whether to enable self select mode, allowing miners can use their own block template. Currently in testing, not for
    production use yet. You must set the environment variable ``RUSTPOOL_DEV`` to a non-blank value to set this to
    ``true``.

**auto_payment_min_balance_atomic_units** - u64
    The minimum balance in atomic units before a miner will get an auto payout, if ``should_do_automatic_payments``
    is true.

**block_notify_host** - str
    The IP to listen to for block notifications. This may be set to ``0.0.0.0`` to accept notifications from all
    addresses.

**block_notify_port** - u16
    The port to listen to for block notifications. This may be set to ``0`` to accept notifications from all ports.

**daemon_rpc_url** - str
    The full url to connect to the Monero RPC at.

**database_connect_timeout_seconds** - u8
    The database connection timeout in seconds.

**database_host** - str
    The host to connect to the database at.

**database_name** - str
    The name of the database to connect to.

**database_password** - str
    The password for the database user.

**database_port** - u16
    The port to connect to the database at.

**database_type** - str
    The type of database to connect to. Currently only ``POSTGRES`` is allowed here.

**database_user** - u16
    The database user to connect as.

**fee_wallet** - str
    The pool fee wallet.

**log_level** - str
    The log level. Must be one of ``TRACE``, ``DEBUG``, ``INFO``, ``WARNING``, ``ERROR``, or ``OFF``.

**log_style** - str
    The log style. Must be one of ``RUSTPOOL`` or ``SYSTEMD``.

**manual_payment_min_balance_atomic_units** - u64
    The minimum balance in atomic units before a miner can request a manual payout.

**max_error_jobs_to_block** - u8
    The number of errored jobs a miner can have in the past ``pool_stats_window_seconds`` before they are refused
    another one.

**max_open_jobs_to_block** - u8
    The number of open jobs a miner can have in the past ``pool_stats_window_seconds`` before they are refused
    another one.

**miner_expected_seconds_per_share** - u16
    This is how often, on average, you would like a miner to produce a share. RustPool will adjust the miner's
    difficulty target based on their average hashrate in the past ``pool_stats_window_seconds`` so that this value
    is targeted for the next job.

**node_role** - str
    This is the role the pool instance should perform. Must be one of ``BACKEND``, ``WORKER``, or ``COMBINED``.
    RustPool is designed so that only one pool node should perform backend duties, so any deployment should respect
    this assumption.

**poll_rpc_interval_seconds** - u16
    How often the backend should poll the Monero RPC for new block templates. Although RustPool is designed to listen
    for block notifications, manual polling is still recommended by the daemon documentation in cases of missed
    notifications.

**pool_fee** - f64
    The pool fee expressed as a floating point number between ``0.0`` and ``1.0`` (So ``1%`` becomes ``0.01``
    and so on).

**pool_listen_host** - str
    The host to listen for miner requests on. This may be set to ``0.0.0.0`` to accept miner requests from
    all addresses.

**pool_listen_port** - u16
    The port to listen for miner requests on. This may be set to ``0`` to accept miner requests from all ports.

**pool_min_difficulty** - u64
    The minimum difficulty for a miner to submit a share.

**pool_nonce_slug** - str
    A slug to add to the block template reserved space. It must be a hexadecimal string that is an even number of
    characters in length and no more than ``2 * pool_reserve_size_bytes`` characters long. In practice, this should
    be significantly shorter than``2 * pool_reserve_size_bytes`` so that sufficient entropy may be introduced into
    the template, to avoid miners mining the same template.

**pool_reserve_size_bytes** - u16
    The number of bytes to request for block template reserved space. This corresponds to the pool nonce space in the
    template and will be filled with the ``pool_nonce_slug`` and the remaining space filled with random hex characters
    to prevent miners from mining on the same template.

**pool_stats_window_seconds** - u16
    The number of seconds to look back when determining miner hashrate, open and error job counts, etc.

**process_payments_timer_seconds** - u16
    How often the backend should process payments in seconds.

**rpc_timeout_seconds** - u8
    The timeout in seconds for connecting to the Monero RPC.

**rx_use_full_memory** - bool
    Sets whether to use the full memory dataset in the RandomX algorithm.

**rx_use_large_pages** - bool
    Sets whether to use large pages in the RandomX algorithm.

**rx_set_secure_flag** - bool
    Sets whether to use secure features in the RandomX algorithm.

**should_do_automatic_payments** - bool
    Whether the backend should do automatic payments once a miner has a balance greater than
    ``auto_payment_min_balance_atomic_units``.

**should_process_payments** - bool
    Whether the backend should process payments or not.

**threads** - u8
    This is the number of threads to use. If you set this to 0, the CPU will be polled and the maximum threads
    available will be used.

**wallet** - str
    The pool wallet address.

**wallet_rpc_url** - str
    The full url to connect to the Monero wallet RPC at.


