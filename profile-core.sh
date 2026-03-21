#!/usr/bin/env bash
set -euo pipefail

runs="${1:-5}"
binary="target/debug/halcyon"
cache_dir="target/.halcyon-cache"

cargo build -p halcyon

echo "== Cold cache (fast mode) =="
hyperfine --runs "$runs" --prepare "rm -rf '$cache_dir'" "$binary build tmp_option_show.hc"

echo "== Warm cache (fast mode) =="
"$binary" build tmp_option_show.hc >/dev/null
hyperfine --warmup 1 --runs "$runs" "$binary build tmp_option_show.hc"

echo "== Warm cache (debug metadata enabled) =="
env HALCYON_DEBUG_INFO=1 "$binary" build tmp_option_show.hc >/dev/null
hyperfine --warmup 1 --runs "$runs" "env HALCYON_DEBUG_INFO=1 $binary build tmp_option_show.hc"

echo "== Warm profile (fast mode) =="
env HALCYON_PROFILE=1 "$binary" build tmp_option_show.hc
