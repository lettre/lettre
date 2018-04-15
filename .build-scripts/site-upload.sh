#!/bin/bash

cd website
make clean && make
sudo pip install ghp-import
ghp-import -n _book
git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages