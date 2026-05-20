#!/bin/bash
# Hoverdo session-start hook.
#
# Hoverdo targets Windows for end-users, but Claude Code on the web runs in
# an ephemeral Ubuntu container. Tauri's build pipeline (and `cargo check`)
# pulls in GTK/WebKit system libs that aren't preinstalled, and pnpm needs to
# fetch frontend deps. We install both so `pnpm check`, `pnpm build`, and
# `cargo check` all work out of the box.
#
# Idempotent: apt skips already-installed packages, `pnpm install` is a no-op
# when the lockfile and node_modules are already in sync.
set -euo pipefail

# Local hosts already have whatever they need; do nothing.
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "$CLAUDE_PROJECT_DIR"

echo "==> Installing Tauri Linux build prerequisites (apt)"
# Best-effort update - some pre-existing third-party PPAs in the base image
# may be unsigned. The packages we actually need come from the standard
# Ubuntu repos, so a partial update is fine.
sudo apt-get update -qq || true
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsoup-3.0-dev \
  build-essential \
  pkg-config

echo "==> Installing pnpm deps"
pnpm install --frozen-lockfile
