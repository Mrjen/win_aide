#!/bin/bash
set -e

echo "Building TailwindCSS..."
npx @tailwindcss/cli -i tailwind.css -o packages/web/assets/tailwind.css --minify
cp packages/web/assets/tailwind.css packages/desktop/assets/tailwind.css
cp packages/web/assets/tailwind.css packages/mobile/assets/tailwind.css
echo "TailwindCSS build complete."
