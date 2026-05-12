#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd -- "${SCRIPT_DIR}/.." && pwd)

sudo install -Dm755 \
    "${REPO_ROOT}/scripts/kay-memory-guard.sh" \
    /usr/local/bin/kay-memory-guard.sh

sudo install -Dm644 \
    "${REPO_ROOT}/systemd/kay-memory-guard.service" \
    /etc/systemd/system/kay-memory-guard.service

sudo systemctl daemon-reload
sudo systemctl enable kay-memory-guard.service
sudo systemctl restart kay-memory-guard.service
sudo systemctl --no-pager --full status kay-memory-guard.service
