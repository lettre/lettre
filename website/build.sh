#!/bin/sh
hugo
lunr-hugo -i "content/**/*.md" -o ../docs/index.json -l toml
