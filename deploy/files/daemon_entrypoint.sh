#!/bin/bash
printf "starting daemon as user %s\n" "${MONERO_USER:=monero_user}"
chown -R "${MONERO_USER}":"${MONERO_USER}" /monero_data
DNS_PUBLIC=tcp gosu "${MONERO_USER}" monerod \
  --rpc-login "${MONERO_DAEMON_RPC_USER:?UNSET}:${MONERO_DAEMON_RPC_PASSWORD:?UNSET}" \
  --non-interactive \
  --block-notify "/bin/bash /block_notify.sh" \
  --data-dir /monero_data \
  --log-file /dev/stdout \
  --log-level 0 \
  --max-log-file-size 0 \
  --p2p-bind-ip "0.0.0.0" \
  --p2p-bind-port 18080 \
  --rpc-bind-ip "0.0.0.0" \
  --rpc-bind-port 18081 \
  --confirm-external-bind \
  --no-igd \
  --db-sync-mode safe \
  --enforce-dns-checkpointing \
  --out-peers 64 \
  --in-peers 256 \
  --limit-rate-up 2048 \
  --limit-rate-down 8192
