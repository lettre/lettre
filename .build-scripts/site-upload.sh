#!/bin/bash

set -xe

cd website
make clean && make
echo "lettre.at" > _book/CNAME
sudo pip install ghp-import
ghp-import -n _book
git push -f https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages