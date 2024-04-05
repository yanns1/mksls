#!/bin/sh

set -e

echo "Compiling..."
cargo build --release 2>/dev/null || (echo "Compilation failed :(" && exit 1)

link="$HOME/.local/bin/mksls"
echo "Making symlink $link"
cd "$(dirname "$0")"
target="$(pwd)/target/release/mksls"
ln -s -i "$target" "$link" || (echo "Symlink creation failed :(" && exit 1)

echo "mksls successfully installed!"
