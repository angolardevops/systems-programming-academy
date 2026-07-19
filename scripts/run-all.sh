#!/usr/bin/env bash
#
# run-all.sh — run every example suite in the Academy with one command.
#
# For each directory under examples/, detect the language and run its tests the
# same way CI does:
#   * Cargo.toml     -> cargo test
#   * go.mod         -> go test ./...
#   * test_*.py      -> python3 -m unittest discover -s . -p 'test_*.py'
#
# Usage:
#   scripts/run-all.sh              # run every suite (tests only)
#   scripts/run-all.sh --full       # also run fmt/lint gates (cargo fmt+clippy,
#                                    #   gofmt+go vet, ruff format+check)
#   scripts/run-all.sh rust         # only Rust suites (also: go | python)
#   scripts/run-all.sh part6        # only dirs whose name contains "part6"
#
# Exit code is non-zero if any suite fails. No arguments needed beyond a working
# Rust (cargo), Go, and Python 3 toolchain; --full also needs clippy and ruff.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 1
EXAMPLES="examples"

FULL=0
FILTER=""
for arg in "$@"; do
  case "$arg" in
    --full) FULL=1 ;;
    rust | go | python) FILTER="lang:$arg" ;;
    *) FILTER="name:$arg" ;;
  esac
done

green() { printf '\033[32m%s\033[0m' "$1"; }
red() { printf '\033[31m%s\033[0m' "$1"; }
dim() { printf '\033[2m%s\033[0m' "$1"; }

pass=0
fail=0
skip=0
failed_names=()

want() { # want <dir> <lang>
  case "$FILTER" in
    "") return 0 ;;
    lang:*) [ "${FILTER#lang:}" = "$2" ] ;;
    name:*) [[ "$1" == *"${FILTER#name:}"* ]] ;;
  esac
}

run_one() { # run_one <name> <lang> <cmd...>
  local name="$1" lang="$2"
  shift 2
  printf '  %-28s %-7s ' "$name" "[$lang]"
  if out=$("$@" 2>&1); then
    green "ok"
    printf '\n'
    pass=$((pass + 1))
  else
    red "FAIL"
    printf '\n'
    echo "$out" | tail -8 | sed 's/^/      /'
    fail=$((fail + 1))
    failed_names+=("$name")
  fi
}

for dir in "$EXAMPLES"/*/; do
  name="$(basename "$dir")"
  if [ -f "$dir/Cargo.toml" ]; then
    want "$name" rust || {
      skip=$((skip + 1))
      continue
    }
    if [ "$FULL" = 1 ]; then
      run_one "$name" rust bash -c "cd '$dir' && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test"
    else
      run_one "$name" rust bash -c "cd '$dir' && cargo test"
    fi
  elif [ -f "$dir/go.mod" ]; then
    want "$name" go || {
      skip=$((skip + 1))
      continue
    }
    if [ "$FULL" = 1 ]; then
      run_one "$name" go bash -c "cd '$dir' && test -z \"\$(gofmt -l .)\" && go vet ./... && go test ./..."
    else
      run_one "$name" go bash -c "cd '$dir' && go test ./..."
    fi
  elif ls "$dir"test_*.py >/dev/null 2>&1; then
    want "$name" python || {
      skip=$((skip + 1))
      continue
    }
    if [ "$FULL" = 1 ]; then
      run_one "$name" python bash -c "cd '$dir' && ruff format --check . && ruff check . && python3 -m unittest discover -s . -p 'test_*.py'"
    else
      run_one "$name" python bash -c "cd '$dir' && python3 -m unittest discover -s . -p 'test_*.py'"
    fi
  else
    skip=$((skip + 1))
  fi
done

printf '\n'
printf '  %s  %s  %s\n' "$(green "$pass passed")" "$(red "$fail failed")" "$(dim "$skip skipped")"
if [ "$fail" -gt 0 ]; then
  printf '  failed: %s\n' "${failed_names[*]}"
  exit 1
fi
