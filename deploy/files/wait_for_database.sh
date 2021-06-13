#!/bin/bash
until pg_isready -h rustpool-db -p 5432 -d rustpool -U postgres; do
  sleep 10
  printf "waiting 10 seconds for database to become ready\n"
done
printf "database ready\n"
