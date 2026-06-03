# Third-Party Assets

## 1mposter ASCII Magician GIF

- Source asset: `yazelix-screen:assets/third_party/ascii_magician_1mposter.gif`
- Runtime source: `yazelix-screen:share/yazelix_screen/ascii_magician_1mposter.gif`
- Cached derivative: `~/.cache/yazelix_screen/ascii_magician_1mposter_frames/`
- Source URL: `https://media4.giphy.com/media/v1.Y2lkPTc5MGI3NjExazh2M3hxMDBjNG5wZnl0MTJkOTQyYjh4bnJudnNvcGZoaW95c3NoYyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/l1Zx0GjghUUF6cGXFk/giphy.gif`
- Creator attribution: `1mposter`
- Yazelix style: `magician`
- Permission: maintainer confirmed Yazelix may use the GIF with attribution on 2026-05-22

The packaged runtime carries the source GIF, not expanded PNG frames and not
ImageMagick. When explicit `magician` playback needs PNG frames, Yazelix uses
existing runtime/cache frames or asks host ImageMagick `magick` to generate the
cache from the packaged GIF. The `random` style skips `magician` when those
assets cannot be resolved or generated.
