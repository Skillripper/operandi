#!/usr/bin/env python3
"""Write the website's guide-level 404 page, public/guide/404.html.

The site serves the nearest 404.html for a missing page. Inside a published
guide folder (test/, latest/, 0.2.0/ ...) that is the folder's own mdBook
404 page, whose script (docs/guide/theme/operandi.js) turns an unclaimed
go/<help-id>/ into a search. Anything else under /guide/ gets this page.

Its job: Help > User Guide in a release build opens /guide/<its version>/
(and /guide/<version>/go/<help-id>/ for context help), but a version's
folder only exists once its release tag has published it. So for
/guide/<folder>/<rest> where <folder> isn't published, this page sends the
browser to /guide/latest/<rest>, or /guide/test/<rest> before the first
release has a guide. go/<help-id>/ paths then reach that guide's redirect,
or its search for an id no page claims.

The list of published folders is written into the page, so run this after
every change to public/guide/ (the publish workflow does).

    python3 tools/guide_404.py website/public/guide

Uses only the standard library.
"""
import json
import sys
from pathlib import Path

PAGE = """<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8">
<title>Operandi User Guide</title>
<meta name="robots" content="noindex">
<script>
// Written by tools/guide_404.py: the guide folders published on this site.
(function () {
    var published = __PUBLISHED__;
    var fallback = __FALLBACK__;
    if (!fallback) return;
    var m = location.pathname.match(/^\\/guide\\/([^\\/]+)(?:\\/(.*))?$/);
    var target = "/guide/" + fallback + "/";
    if (m && published.indexOf(m[1]) >= 0) {
        // A published folder normally answers with its own 404 page;
        // if not, its front page is the best guess.
        target = "/guide/" + m[1] + "/";
    } else if (m) {
        // An unpublished version: the same page in the fallback guide.
        target += m[2] || "";
    }
    location.replace(target + location.search + location.hash);
})();
</script>
</head>
<body><p>This page isn't in the guide. Go to the
<a href="/guide/">Operandi User Guide</a>.</p></body></html>
"""


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: guide_404.py <public/guide folder>", file=sys.stderr)
        return 2
    guide = Path(sys.argv[1])
    published = sorted(p.name for p in guide.iterdir() if p.is_dir())
    fallback = next((f for f in ("latest", "test") if f in published), None)
    page = PAGE.replace("__PUBLISHED__", json.dumps(published)).replace(
        "__FALLBACK__", json.dumps(fallback)
    )
    (guide / "404.html").write_text(page, encoding="utf-8")
    print(f"guide/404.html: unpublished folders go to {fallback or 'nowhere'}/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
