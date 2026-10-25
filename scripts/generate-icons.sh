#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
icon_dir="$repo_root/src-tauri/icons"
source_icon="$icon_dir/icon.png"

if ! command -v magick >/dev/null 2>&1; then
  printf '%s\n' 'ImageMagick 7 (magick) is required to generate icon derivatives.' >&2
  exit 1
fi

if [[ ! -f "$source_icon" ]]; then
  printf 'Source icon not found: %s\n' "$source_icon" >&2
  exit 1
fi

temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT

render_png() {
  local size="$1"
  local output="$2"
  magick "$source_icon" \
    -filter Lanczos \
    -resize "${size}x${size}" \
    -strip \
    "PNG32:$output"
}

render_png 32 "$icon_dir/32x32.png"
render_png 128 "$icon_dir/128x128.png"
render_png 256 "$icon_dir/128x128@2x.png"
render_png 512 "$icon_dir/512x512.png"

for size in 16 24 32 48 64 128 256; do
  render_png "$size" "$temporary_dir/${size}.png"
done

magick \
  "$temporary_dir/16.png" \
  "$temporary_dir/24.png" \
  "$temporary_dir/32.png" \
  "$temporary_dir/48.png" \
  "$temporary_dir/64.png" \
  "$temporary_dir/128.png" \
  "$temporary_dir/256.png" \
  "$icon_dir/icon.ico"

printf 'Generated Linux PNG and Windows ICO derivatives from %s\n' "$source_icon"
