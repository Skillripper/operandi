#!/bin/sh
# Schedules scripts/nightly.sh with launchd (macOS's built-in scheduler)
# for 02:30 every night. If the Mac is asleep then, launchd runs it when it
# wakes. Uses no Claude.
#
#   scripts/nightly-install.sh            install (or update) the schedule
#   scripts/nightly-install.sh remove     remove it
#   scripts/nightly-install.sh --test-builds
#                                         also publish a test build each
#                                         night main changed and the tests
#                                         passed (scripts/nightly-test-build.sh)
#
# With --test-builds, run it from a Terminal where OPERANDI_WINDOWS_SSH and
# OPERANDI_LINUX_SSH are set: their values are copied into the schedule
# (one left unset leaves that platform out).
#
# The schedule runs the nightly.sh of the main checkout, so it always tests
# with main's own script.
set -eu
LABEL=com.operandi.nightly
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
REPO=$(cd "$(dirname "$0")/.." && git rev-parse --path-format=absolute --git-common-dir)
REPO=$(dirname "$REPO")
OUT="$HOME/Developer/Operandi-Test-Results"

TEST_BUILDS=0
[ "${1:-}" = --test-builds ] && TEST_BUILDS=1
ENV=""
if [ "$TEST_BUILDS" = 1 ]; then
    ENV="    <key>EnvironmentVariables</key>
    <dict>
        <key>OPERANDI_NIGHTLY_TEST_BUILD</key><string>1</string>
        <key>OPERANDI_WINDOWS_SSH</key><string>${OPERANDI_WINDOWS_SSH:-}</string>
        <key>OPERANDI_LINUX_SSH</key><string>${OPERANDI_LINUX_SSH:-}</string>
    </dict>"
fi

launchctl bootout "gui/$(id -u)/$LABEL" 2>/dev/null || true
if [ "${1:-}" = remove ]; then
    rm -f "$PLIST"
    echo "Nightly run removed."
    exit 0
fi

mkdir -p "$HOME/Library/LaunchAgents" "$OUT"
cat >"$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>$LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/sh</string>
        <string>$REPO/scripts/nightly.sh</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict><key>Hour</key><integer>2</integer><key>Minute</key><integer>30</integer></dict>
    <!-- Full speed: with LowPriorityIO and Nice 10 macOS throttled the
         run so hard that clippy took 4.5 hours instead of minutes
         (2026-11-19). -->
    <key>ProcessType</key><string>Standard</string>
$ENV
    <key>StandardOutPath</key><string>$OUT/launchd.log</string>
    <key>StandardErrorPath</key><string>$OUT/launchd.log</string>
</dict>
</plist>
EOF
launchctl bootstrap "gui/$(id -u)" "$PLIST"
echo "Nightly run scheduled for 02:30. Results: $OUT/latest.md"
if [ "$TEST_BUILDS" = 1 ]; then
    echo "Test builds: on, after passing tests (Windows: ${OPERANDI_WINDOWS_SSH:-off}, Linux: ${OPERANDI_LINUX_SSH:-off}). Result: $OUT/latest-test-build.md"
fi
echo "Run it now with: launchctl kickstart gui/$(id -u)/$LABEL"
