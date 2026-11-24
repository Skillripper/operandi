#!/usr/bin/env python3
"""Write the User Guide's go/<help-id>/ redirect pages into a built book.

Every guide page starts with a header comment naming the help ids it
covers (docs/guide/RULES.md):

    <!-- help-ids: Bevel, poly.bevel ; template: tool ; since: 0.1.0 -->

For each id this writes book/go/<id>/index.html, which sends the browser
on to that page, so the app can open /guide/<version>/go/<help-id>/ for a
control without knowing where its page is. It also writes go/help-ids.json
(id -> page). An id no page claims has no redirect: the site serves the
guide's 404 page there, and theme/operandi.js turns that into a search of
the guide for the id.

Run after `mdbook build` (scripts/guide-build.sh does both). Uses only the
standard library.
"""
import html
import json
import re
import sys
from pathlib import Path

HEADER = re.compile(r"<!--\s*help-ids:\s*([^;>]*?)\s*(?:;[^>]*)?-->")


def help_ids(md: Path):
    """The ids in a page's header comment (first line), or []."""
    with md.open(encoding="utf-8") as f:
        first = f.readline()
    m = HEADER.match(first.strip())
    if not m:
        return []
    return [i.strip() for i in m.group(1).split(",") if i.strip()]


def page_html(md: Path, src: Path) -> str:
    """The built page for a source page: README.md is index.html."""
    rel = md.relative_to(src).with_suffix(".html")
    if rel.name == "README.html":
        rel = rel.with_name("index.html")
    return rel.as_posix()


def collect(src: Path):
    """id -> page, failing on an id two pages claim or one that can't be a
    folder name."""
    found, errors = {}, []
    for md in sorted(src.rglob("*.md")):
        if md.name == "SUMMARY.md":
            continue
        page = page_html(md, src)
        for hid in help_ids(md):
            if "/" in hid or "\\" in hid or hid in (".", "..") or hid.startswith("."):
                errors.append(f"{md}: help id {hid!r} can't be a folder name")
            elif hid in found and found[hid] != page:
                errors.append(f"{md}: help id {hid!r} is already on {found[hid]}")
            else:
                found[hid] = page
    return found, errors


REDIRECT = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<meta name="robots" content="noindex">
<meta http-equiv="refresh" content="0; url={target}">
<link rel="canonical" href="{target}">
<script>location.replace("{target_js}" + location.hash);</script>
</head>
<body><p>This help page has moved to <a href="{target}">{title}</a>.</p></body>
</html>
"""


def write(book: Path, ids):
    go = book / "go"
    for hid, page in sorted(ids.items()):
        target = "../../" + page
        d = go / hid
        d.mkdir(parents=True, exist_ok=True)
        (d / "index.html").write_text(
            REDIRECT.format(
                title=html.escape(hid),
                target=html.escape(target, quote=True),
                target_js=json.dumps(target)[1:-1],
            ),
            encoding="utf-8",
        )
    go.mkdir(parents=True, exist_ok=True)
    (go / "help-ids.json").write_text(
        json.dumps(dict(sorted(ids.items())), indent=1) + "\n", encoding="utf-8"
    )


def main():
    guide = Path(sys.argv[1] if len(sys.argv) > 1 else "docs/guide")
    src, book = guide / "src", guide / "book"
    if not (book / "index.html").exists():
        sys.exit(f"{book} has no built guide; run mdbook build first")
    ids, errors = collect(src)
    if errors:
        sys.exit("\n".join(errors))
    write(book, ids)
    print(f"go/: {len(ids)} help ids")


if __name__ == "__main__":
    main()
