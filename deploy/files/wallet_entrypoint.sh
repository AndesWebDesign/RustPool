#!/bin/bash
. /wait_for_daemon.sh
printf "starting wallet rpc as user %s\n" "${MONERO_USER:=monero_user}"
chown -R "${MONERO_USER}":"${MONERO_USER}" /wallet_data
gosu "${MONERO_USER}" monero-wallet-rpc \
  --non-interactive \
  --log-file /dev/stdout \
  --max-log-file-size 0 \
  --wallet-file /wallet_data/pool_wallet \
  --password "${POOL_WALLET_PASSWORD:?UNSET}" \
  --rpc-bind-ip "0.0.0.0" \
  --rpc-bind-port 18082 \
  --confirm-external-bind \
  --rpc-login "${MONERO_WALLET_RPC_USER:?UNSET}:${MONERO_WALLET_RPC_PASSWORD:?UNSET}" \
  --daemon-address "monero-daemon:18081" \
  --daemon-login "${MONERO_DAEMON_RPC_USER:?UNSET}:${MONERO_DAEMON_RPC_PASSWORD:?UNSET}" \
  --trusted-daemon
