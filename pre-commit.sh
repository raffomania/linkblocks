#!/bin/bash

just start-database

if ! cargo bin sqlx-cli prepare --check; then
    cargo bin sqlx-cli prepare
    exit 1
fi
