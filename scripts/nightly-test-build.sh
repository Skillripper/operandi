#!/bin/sh
# The nightly test build: puts GitHub's newest main on the rolling "Test
# build" pre-release (scripts/test-build.sh), but only when main has
# changed since the build that is up there now. Apps on the Test channel
# then offer it at their next launch, so nobody has to be told.
#
#   scripts/nightly-test-build.sh            build and upload if main moved
#   scripts/nightly-test-build.sh --force    build and upload anyway
#
# scripts/nightly.sh runs it after the night's tests pass, when the
# schedule was installed with `scripts/nightly-install.sh --test-builds`.
# It can also be run by hand. Needs what test-build.sh needs: the GitHub CLI
# signed in, and OPERANDI_WINDOWS_SSH / OPERANDI_LINUX_SSH for those two
# platforms (a platform whose variable isn't set is left out, and its old
# file stays up).
#
# Log: ~/Developer/Operandi-Test-Results/test-build/<date>.log, and
# latest-test-build.md says what happened. A failure also shows a macOS
# notification, so it isn't missed.
set -u
PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
export PATH

REPO=$(cd "$(dirname "$0")/.." && git rev-parse --path-format=absolute --git-common-dir)
REPO=$(dirname "$REPO")
OUT="${OPERANDI_NIGHTLY_OUT:-$HOME/Developer/Operandi-Test-Results}"
STAMP=$(date +%Y-%m-%d)
LOG="$OUT/test-build/$STAMP.log"
STATUS="$OUT/latest-test-build.md"
mkdir -p "$OUT/test-build"

FORCE=0
[ "${1:-}" = --force ] && FORCE=1

note() { # title, message: a macOS notification (best effort)
    command -v osascript >/dev/null 2>&1 || return 0
    osascript -e "display notification \"$2\" with title \"$1\"" >/dev/null 2>&1 || true
}
finish() { # result line, exit code
    printf '# Test build %s: %s\n\nLog: `%s`\n' "$STAMP" "$1" "$LOG" >"$STATUS"
    printf '%s\n' "$1" >>"$LOG"
    exit "$2"
}

{
    echo "== Nightly test build, $(date)"
} >"$LOG"

# What main is on GitHub now, and what the test build was made from.
if ! git -C "$REPO" fetch --quiet origin main >>"$LOG" 2>&1; then
    note "Operandi test build" "Couldn't reach GitHub; no test build tonight."
    finish "failed: couldn't fetch main from GitHub" 1
fi
MAIN=$(git -C "$REPO" rev-parse FETCH_HEAD)
GH_REPO=$(gh repo view "$(git -C "$REPO" remote get-url origin)" --json nameWithOwner -q .nameWithOwner 2>>"$LOG")
export GH_REPO
BODY=$(gh release view test-build --json body -q .body 2>/dev/null || true)
up() { printf '%s\n' "$BODY" | grep '^<!-- operandi-build ' | head -1 | tr ' ' '\n' | sed -n "s/^$1=//p"; }

# Platforms this Mac can build tonight: the Mac always, the others when
# their machine is set.
PLATFORMS=mac
[ -n "${OPERANDI_WINDOWS_SSH:-}" ] && PLATFORMS="$PLATFORMS,windows"
[ -n "${OPERANDI_LINUX_SSH:-}" ] && PLATFORMS="$PLATFORMS,linux"

if [ "$FORCE" = 0 ]; then
    STALE=0
    for p in $(printf '%s' "$PLATFORMS" | tr ',' ' '); do
        [ "$(up "$p")" = "$MAIN" ] || STALE=1
    done
    if [ "$STALE" = 0 ]; then
        finish "skipped: the test build is already main ($(printf '%.8s' "$MAIN"))" 0
    fi
fi

echo "Building main at $(printf '%.8s' "$MAIN") for $PLATFORMS" >>"$LOG"
if sh "$REPO/scripts/test-build.sh" main --platforms "$PLATFORMS" >>"$LOG" 2>&1; then
    finish "uploaded main $(printf '%.8s' "$MAIN") ($PLATFORMS)" 0
fi
note "Operandi test build failed" "Tonight's test build didn't upload. See latest-test-build.md."
finish "**FAILED** (see the log)" 1
