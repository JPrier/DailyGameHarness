#!/usr/bin/env bash
set -euo pipefail
cargo run -p daily_game_tools -- prepare-static-build
cd web && npm run build
