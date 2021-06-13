#!/bin/bash
. /wait_for_daemon.sh
. /wait_for_database.sh
printf "starting rustpool as user %s\n" "${RUSTPOOL_USER:=rustpool_user}"
chown "${RUSTPOOL_USER}":"${RUSTPOOL_USER}" /etc/rustpool
gosu "${RUSTPOOL_USER}" rustpool -c /etc/rustpool/config.json
