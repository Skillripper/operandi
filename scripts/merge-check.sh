#!/bin/sh
# The check before proposing a merge (testing plan item 1, Jason
# 2026-11-18): the full tier from AGENTS.md on this branch with main
# merged in, plus the Cycles and USD tests when their libraries are built.
# Those tests only compile with `--features cycles` / `usd`, so plain
# `cargo test` and CI never run them.
#
#   scripts/merge-check.sh
#
# Exits non-zero on the first failing step. It changes nothing in this
# checkout or its branch: if main isn't merged in yet, it says so and stops.
#
# Where it builds (merge-check cache, 2026-11-26). Every branch is checked
# in one shared merge checkout, ../Operandi-merge-check next to the main
# checkout, with its own warm build folder, instead of in the branch's
# worktree. Each run checks out this branch's commit there (plus any
# uncommitted changes to tracked files) and runs the steps in it. Git
# rewrites only the files that differ from the last branch checked, so
# Cargo rebuilds only what those files touch, and the Cycles/USD/Python
# builds are warm too. A fresh worktree's first merge check otherwise
# rebuilds every Operandi crate, its C++ and every feature set.
# - Correct: the merge checkout's files are exactly this commit's, and a
#   file whose content changed gets a new time, so Cargo's usual checks
#   (file times) can't reuse another branch's results for it.
# - One run at a time: a lock in its .git folder; a second run waits.
# - Files that git doesn't track (new files you haven't `git add`ed) are
#   not part of the check, as they wouldn't be part of the merge.
# - OPERANDI_MERGE_CHECK_DIR=<folder> uses another merge checkout;
#   OPERANDI_MERGE_CHECK_DIR=here checks in this checkout, as before.
# - The build folder is deleted and starts over when it grows past
#   OPERANDI_MERGE_CHECK_MAX_GB (default 150).
set -eu
cd "$(dirname "$0")/.."
CARGO_BIN="${CARGO_BIN:-cargo}"
SRC=$(pwd -P)

if [ -z "${OPERANDI_MERGE_CHECK_LOCKED:-}" ]; then
    if ! git merge-base --is-ancestor main HEAD; then
        echo "merge-check: main has commits this branch lacks; merge main in first (git merge main)." >&2
        exit 1
    fi
    if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
        echo "merge-check: note, uncommitted changes to tracked files are included in this check."
    fi
fi

# The spec-backed tests (oracles, cases) build only when the private specs
# checkout is found (crates/operandi-core/build_specs.rs). A merge must run
# them, so a missing checkout stops here instead of skipping them quietly.
SPECS="${OPERANDI_SPECS:-../Operandi-specs}"
if [ ! -d "$SPECS/specifications" ]; then
    echo "merge-check: no private specs at $SPECS; clone them there or set OPERANDI_SPECS." >&2
    exit 1
fi

# The main checkout is the first entry `git worktree list` prints.
MAIN=$(git worktree list --porcelain | sed -n '1s/^worktree //p')
MC="${OPERANDI_MERGE_CHECK_DIR:-$(dirname "$MAIN")/Operandi-merge-check}"

if [ "$MC" != here ] && [ "$SRC" != "$(cd "$MC" 2>/dev/null && pwd -P)" ]; then
    if [ -z "${OPERANDI_MERGE_CHECK_LOCKED:-}" ]; then
        if [ ! -d "$MC/.git" ]; then
            # Its own small git folder, borrowing the main repository's
            # objects (every worktree's commits are there), so checking out
            # a commit never touches the main repository.
            echo "merge-check: making the merge checkout $MC (once)."
            git clone -q --shared --no-checkout "$MAIN" "$MC"
        fi
        # Hold the lock for the rest of the run (it's dropped when this
        # process ends, however it ends). perl's flock is on macOS and Linux.
        OPERANDI_MERGE_CHECK_LOCKED=1 exec perl -MFcntl=:flock -e '
            $^F = 255; # keep the lock open across exec
            open(my $lock, ">>", shift) or die "merge-check: lock: $!\n";
            unless (flock($lock, LOCK_EX | LOCK_NB)) {
                print "merge-check: another merge check is running; waiting for it.\n";
                flock($lock, LOCK_EX) or die "merge-check: lock: $!\n";
            }
            exec @ARGV or die "merge-check: $!\n";
        ' "$MC/.git/merge-check.lock" sh "$0"
    fi

    COMMIT=$(git rev-parse HEAD)
    PATCH=$(mktemp)
    trap 'rm -f "$PATCH"' EXIT
    git diff --binary HEAD >"$PATCH"
    echo "merge-check: checking $(git rev-parse --abbrev-ref HEAD) ($(git rev-parse --short HEAD)) in $MC"
    # -f also undoes the last run's uncommitted changes; clean removes
    # untracked files left from other branches (ignored ones, like the
    # build output and development/, stay).
    if ! git -C "$MC" checkout -q -f --detach "$COMMIT" 2>/dev/null; then
        # The commit it had checked out may have been pruned since. Start
        # its files over (the build folder stays; it's keyed by the path).
        echo "merge-check: refreshing the merge checkout's files."
        find "$MC" -mindepth 1 -maxdepth 1 ! -name .git ! -name target ! -name development -exec rm -rf {} +
        rm -f "$MC/.git/index"
        git -C "$MC" update-ref --no-deref HEAD "$COMMIT"
        git -C "$MC" reset -q --hard
    fi
    git -C "$MC" clean -q -f -d
    if [ -s "$PATCH" ]; then
        git -C "$MC" apply --whitespace=nowarn "$PATCH"
    fi
    # The built Cycles, USD and Python this checkout tests with (git-ignored).
    for lib in cycles/install usd/install python; do
        if [ -d "development/$lib" ]; then
            mkdir -p "$MC/development/$lib"
            rsync -a --delete "development/$lib/" "$MC/development/$lib/"
        else
            rm -rf "${MC:?}/development/$lib"
        fi
    done
    OPERANDI_SPECS="$(cd "$SPECS" && pwd -P)"
    export OPERANDI_SPECS
    cd "$MC"
    SPECS="$OPERANDI_SPECS"

    # Bound its disk use: start the build folder over past the limit.
    BUILD_DIR=$("$CARGO_BIN" metadata --format-version 1 --no-deps --offline |
        python3 -c 'import json, sys; m = json.load(sys.stdin); print(m.get("build_directory") or m["target_directory"])')
    MAX_GB="${OPERANDI_MERGE_CHECK_MAX_GB:-150}"
    if [ -d "$BUILD_DIR" ]; then
        USED_GB=$(($(du -sk "$BUILD_DIR" | cut -f1) / 1048576))
        echo "merge-check: build folder $BUILD_DIR, ${USED_GB} GB (limit ${MAX_GB} GB)."
        if [ "$USED_GB" -gt "$MAX_GB" ]; then
            echo "merge-check: over the limit; starting the build folder over (this run builds cold)."
            rm -rf "${BUILD_DIR:?}"
        fi
    fi
fi

step() { echo; echo "== $*"; }

step "fmt"
"$CARGO_BIN" fmt --check
step "clippy"
"$CARGO_BIN" clippy --locked --workspace --all-targets -- -D warnings
step "tests"
scripts/test-all.sh

# The feature-gated tests, each only where its library is built.
if [ -d development/cycles/install/lib ]; then
    step "Cycles tests (--features cycles)"
    "$CARGO_BIN" test --locked --workspace --features operandi-app/cycles
else
    echo; echo "== Cycles tests skipped: no development/cycles (scripts/build-cycles.sh)"
fi
if [ -d development/usd/install/lib ]; then
    step "USD tests (--features usd)"
    "$CARGO_BIN" test --locked --workspace --features operandi-app/usd
else
    echo; echo "== USD tests skipped: no development/usd (scripts/build-usd.sh)"
fi
# Python scripting: the bundled CPython (scripts/fetch-python.sh), else a
# python3 on PATH for development.
PY_FOR_TESTS=""
if [ -x development/python/python/bin/python3 ]; then
    PY_FOR_TESTS="$(cd development/python/python/bin && pwd -P)/python3"
elif command -v python3 >/dev/null 2>&1; then
    PY_FOR_TESTS="$(command -v python3)"
fi
if [ -f development/python/pyo3-config.txt ]; then
    step "Python tests (--features python, bundled CPython)"
    PYO3_CONFIG_FILE="$(cd development/python && pwd -P)/pyo3-config.txt" \
        OPERANDI_PYTHON_BUILD_HOME="$(cd development/python/python && pwd -P)" \
        "$CARGO_BIN" test --locked --workspace --features operandi-app/python
elif [ -n "$PY_FOR_TESTS" ]; then
    step "Python tests (--features python, $PY_FOR_TESTS)"
    PYO3_PYTHON="$PY_FOR_TESTS" "$CARGO_BIN" test --locked --workspace --features operandi-app/python
else
    echo; echo "== Python tests skipped: no Python (scripts/fetch-python.sh)"
fi
echo; echo "merge-check: all passed."
