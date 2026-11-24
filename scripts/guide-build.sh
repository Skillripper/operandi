#!/bin/sh
# Build the User Guide (docs/guide) into docs/guide/book: the clean-room
# name check, mdBook, then the go/<help-id>/ redirect pages the app's help
# links use (tools/guide_redirects.py). Needs mdbook on PATH
# (brew install mdbook, or cargo install mdbook).
#
#   scripts/guide-build.sh
#
# MDBOOK_OUTPUT__HTML__SITE_URL=/guide/<version>/ sets where the book is
# served, so its 404 page finds its styles (the publish workflow sets it).
set -eu
cd "$(dirname "$0")/.."
scripts/guide-name-check.sh
mdbook build docs/guide
python3 tools/guide_redirects.py docs/guide
