<!-- help-ids: operandi.online.refresh, operandi.online.download, operandi.online.clearCache ; template: concept ; since: 0.1.0 -->
# Online libraries

The Preset Browser can show free asset libraries from the web next to
your own folders: HDRIs for lighting, materials and models. You browse
them with thumbnails, then double-click or drag one onto the scene like
any other preset. Each asset downloads the first time you use it and
stays on your computer after that.

Two libraries are available: **Poly Haven** (HDRIs, textures and models)
and **ambientCG** (HDRIs and materials). Every asset in both is CC0, so
you can use it for anything, including paid work, without credit.

## Turn a library on

Every library is off until you turn it on. While it's off, Operandi never
contacts it and it doesn't appear in the browser.

1. Open **Preferences** and go to **Defaults > Application**.
2. In **Online Libraries**, tick **Poly Haven**, **ambientCG** or both.
3. Open the Preset Browser (F6). The library appears in the folder tree
   after **Basics**, with a folder for each kind of asset and one for each
   category inside.

The first time, the library's list of assets (its catalogue) is fetched.
That takes a few seconds; the folder says so while it happens.

## Use an asset

- **Double-click** a tile, or press Enter: an HDRI becomes the
  environment's image, a material loads into the selected material group,
  and a model is added to the scene.
- **Drag** a material onto a mesh to put it on those polygons, or drag an
  HDRI or model into a 3D view, as with any preset.

If the asset isn't downloaded yet, the status line says *Downloading…*,
the tile shows how far it has got, and the asset loads as soon as it
lands. A tile marked **online** isn't downloaded; one marked **2K** (for
example) has a copy at that size.

## Pick a size

Assets download at the **Download Resolution** in Preferences (2K to
start with), or the nearest size the library offers below it. To get a
different size, right-click a tile and choose **Download At**. The menu
also has **Remove Downloaded Files**, **Show Downloaded Files** and
**Open Web Page**, which opens the asset's page on the library's site.

Big sizes are big downloads: an 8K HDRI can be a few hundred megabytes.

## What is stored, and where

Everything goes in the **Online Libraries** folder in Operandi's data
folder. **Preferences > Defaults > Application > Online Libraries** shows
where it is and how much it holds.

- The catalogue: a small list of the library's assets.
- Thumbnails: fetched only for the tiles you scroll past.
- Downloaded files: only for assets you used or asked for.

Whatever is there keeps working offline. **Clear Cache** deletes the
downloads and thumbnails (a library that's off loses its whole folder).

The catalogue is fetched again once a week. **Catalogue Refresh** in
Preferences changes that to monthly or only when you ask, and **Refresh
Catalogue** above the library's tiles fetches it straight away.

Operandi sends nothing about you: there's no account, and each request
only names Operandi and its version.

## Credits

The browser shows each library's credit above its tiles, and every tile's
tooltip lists its licence and authors. Poly Haven's credit is there because
its terms ask for one; ambientCG doesn't ask, but gets one anyway.

## For scripts

- `operandi.online.refresh [source:polyhaven|ambientcg]` fetches
  catalogues again.
- `operandi.online.download source: kind:hdri|texture|model id:
  [resolution:2] [load:true]` downloads an asset, and loads it with
  `load:true`.
- `operandi.online.clearCache [source:]` clears the cache.
- `pref.value online.poly_haven true` turns a library on.

The MCP bridge's `online_library` tool does the same, and lists assets.

## Related

- [Getting started](../getting-started.md)
