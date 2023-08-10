#!/usr/bin/env bash

npx vite build && sed -i 's|/assets|assets|g' dist/index.html
