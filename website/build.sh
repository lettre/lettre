#!/bin/sh
hugo
lunr-hugo -i "content/**/*.md" -o ../docs/json/search.json -l toml
