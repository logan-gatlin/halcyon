#!/usr/bin/env bash
set -euo pipefail

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "error: this script must be run inside a git repository" >&2
  exit 1
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is not clean; commit or stash changes first" >&2
  exit 1
fi

switch_to_branch() {
  local branch="$1"
  local remote_ref="origin/${branch}"

  if git show-ref --verify --quiet "refs/heads/${branch}"; then
    git switch "${branch}" >/dev/null
  else
    git switch --track -c "${branch}" "${remote_ref}" >/dev/null
  fi
}

starting_ref="$(git symbolic-ref --short -q HEAD || true)"

git fetch origin main stable

switch_to_branch main
git pull --ff-only origin main

mapfile -t cargo_files < <(git ls-files -- "Cargo.toml" "**/Cargo.toml")
if [[ "${#cargo_files[@]}" -eq 0 ]]; then
  echo "error: no Cargo.toml files found" >&2
  exit 1
fi

current_year="$(date +%Y)"
current_month="$((10#$(date +%m)))"

new_version="$(
  python - "${current_year}" "${current_month}" "${cargo_files[@]}" <<'PY'
import re
import sys
from pathlib import Path

year = int(sys.argv[1])
month = int(sys.argv[2])
files = [Path(path) for path in sys.argv[3:]]

version_pattern = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
line_pattern = re.compile(r'(\s*version\s*=\s*")(.*?)(".*)')


def load_package_version(path: Path):
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    in_package = False

    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_package = stripped == "[package]"
            continue

        if in_package:
            match = line_pattern.fullmatch(line.rstrip("\n"))
            if match:
                newline = "\n" if line.endswith("\n") else ""
                return lines, index, match.group(2), match.group(1), match.group(3), newline

    raise SystemExit(f"error: could not find [package].version in {path}")


records = []
max_counter = 0

for path in files:
    lines, index, version, prefix, suffix, newline = load_package_version(path)
    version_match = version_pattern.fullmatch(version)
    if version_match is None:
        raise SystemExit(
            f"error: expected version format <year>.<month>.<counter> in {path}, got {version!r}"
        )

    version_year = int(version_match.group(1))
    version_month = int(version_match.group(2))
    version_counter = int(version_match.group(3))

    if version_year == year and version_month == month:
        max_counter = max(max_counter, version_counter)

    records.append((path, lines, index, prefix, suffix, newline))

new_version = f"{year}.{month}.{max_counter + 1}"

for path, lines, index, prefix, suffix, newline in records:
    lines[index] = f"{prefix}{new_version}{suffix}{newline}"
    path.write_text("".join(lines), encoding="utf-8")

print(new_version)
PY
)"

git add "${cargo_files[@]}"
git commit -m "release: bump cargo versions to ${new_version}"
git push origin main

switch_to_branch stable
git pull --ff-only origin stable
git merge main -X theirs --no-edit
git push origin stable

stable_sha="$(git rev-parse --short HEAD)"

if [[ -n "${starting_ref}" && "${starting_ref}" != "stable" ]]; then
  git switch "${starting_ref}" >/dev/null
fi

echo "Bumped Cargo.toml versions to ${new_version}, merged main into stable, and published origin/stable@${stable_sha}"
