#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
outdir="${2:-dist}"

echo "Building release binary..."
cargo build --release

bin_name="lerobot-servo-adjust"
exe="target/release/${bin_name}"

if [[ -z "$version" ]]; then
  version=$(grep -m1 '^version\s*=\s*"' Cargo.toml | sed -E 's/.*"(.*)"/\1/')
fi

arch=$(uname -m)
case "$(uname -s)" in
  Linux) platform="linux";;
  Darwin) platform="macos";;
  MINGW*|MSYS*|CYGWIN*) platform="windows"; exe+=".exe";;
  *) platform="unknown";;
esac

bundle="${bin_name}-${version}-${platform}-${arch}"
root="${outdir}/${bundle}"
rm -rf "$root"
mkdir -p "$root/bin" "$root/templates"

cp "$exe" "$root/bin/"
cp -r templates/* "$root/templates/"
if [[ -d huggingface ]]; then cp -r huggingface "$root/"; fi
cp -f README.md DEVELOP.md GUIDE.md "$root/" 2>/dev/null || true

mkdir -p "$outdir"
zip_path="$outdir/${bundle}.zip"
rm -f "$zip_path"
(cd "$root" && zip -r "../${bundle}.zip" . >/dev/null)

echo "Packed: ${zip_path}"

