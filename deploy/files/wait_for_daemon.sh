#!/bin/bash
daemon_synced() {
  if STATUS_CHECK=$(curl -s \
    -u "${MONERO_DAEMON_RPC_USER:?UNSET}:${MONERO_DAEMON_RPC_PASSWORD:?UNSET}" --digest \
    -X POST \
    http://monero-daemon:18081/json_rpc \
    -d '{"jsonrpc":"2.0","id":"0","method":"sync_info"}' \
    -H 'Content-Type: application/json'); then
    TARGET_HEIGHT=$(echo "$STATUS_CHECK" | jq '.result.target_height' 2>/dev/null)
    HEIGHT=$(echo "$STATUS_CHECK" | jq '.result.height' 2>/dev/null)
    if [ -z "$TARGET_HEIGHT" ] || [ -z "$HEIGHT" ]; then
        return 1;
    fi
    if [ "$TARGET_HEIGHT" -le "$HEIGHT" ]; then
      return 0
    fi
  fi
  return 1
}
while ! daemon_synced; do
  sleep 10
  printf "waiting 10 seconds for daemon to sync\n"
done
printf "daemon synced\n"
