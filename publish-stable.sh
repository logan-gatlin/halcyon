#!/usr/bin/env bash
set -euo pipefail

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "error: this script must be run inside a git repository" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is not clean; commit or stash changes first" >&2
  exit 1
fi

starting_ref="$(git rev-parse --abbrev-ref HEAD)"

git fetch origin main stable
git checkout stable
git pull --ff-only origin stable
git merge origin/main -X theirs --no-edit
git push origin stable

stable_sha="$(git rev-parse --short HEAD)"

if [[ "${starting_ref}" != "stable" ]]; then
  git checkout "${starting_ref}"
fi

echo "Merged origin/main into stable and published origin/stable@${stable_sha}"
