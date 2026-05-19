#!/usr/bin/env bash
set -euo pipefail
aws s3 sync web/dist "s3://${DAILY_GAMES_BUCKET}"
