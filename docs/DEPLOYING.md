CI builds `dist/web/` and publishes it to GitHub Pages on every push to `main`.
One-time setup: repo **Settings → Pages → Source: "GitHub Actions"**. Releases
also carry native binaries and a zipped web bundle.

Because `CassidyPrather/cassidyprather.github.io` serves wirenook.net, this
repository's Pages site is <https://wirenook.net/game-prototypes/>. The site
links to it in one place, its `[params.games]` in `hugo.toml`. The path
follows the repository's name, and GitHub does not redirect Pages after a
rename. Renaming this repo (to `games`, say) therefore means changing that
line, leaving a redirect stub on the site at the old path, and doing the
renames listed in [NEW_GAME.md](NEW_GAME.md#rename).

Or skip Pages entirely and copy `dist/web/` into a static site. It is a folder of files with no external requests; it will run
anywhere you can put a folder of files.
