================
Configuration
================

RustPool is configured through configuration files and environment variables. You may set the configuration via file
alone, environment variables alone, or file with environment variable overrides. RustPool supports files in JSON, YAML,
and TOML formats, though it expects a standard ending for each. All fields in config files are lower snake case, whereas
environment variables are upper (screaming) snake case, with the prefix ``RUSTPOOL_`` attached (e.g. to override the
value of the file field ``fee_wallet``, you would set the environment variable ``RUSTPOOL_FEE_WALLET``). You must
include all fields in the config file or environment variables, as RustPool does not have any internal defaults for
these values. For detailed information on individual configuration values, please see the
:ref:`Fields Reference<fields-reference>`.
