#!/bin/bash

set -xe

cd website
make clean && make
cp static/images/favicon.png _book/html/favicon.png
echo "lettre.at" > _book/html/CNAME
sudo pip install ghp-import
ghp-import -n _book/html
git push -f https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
