#!/bin/bash
set -e

echo "Building TailwindCSS..."
npx @tailwindcss/cli -i tailwind.css -o packages/desktop/assets/tailwind.css --minify
echo "TailwindCSS build complete."
