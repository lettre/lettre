#!/bin/bash

set -o errexit

if [ "$TRAVIS_RUST_VERSION" != "stable" ] || [ "$TRAVIS_PULL_REQUEST" != "false" ]; then
    exit 0
fi

cargo clean
cargo doc --no-deps

git clone --branch gh-pages "https://$GH_TOKEN@github.com/${TRAVIS_REPO_SLUG}.git" deploy_docs
cd deploy_docs

git config user.email "contact@amousset.me"
git config user.name "Alexis Mousset"

if [ "$TRAVIS_BRANCH" == "master"  ]; then
    rm -rf master
    mv ../target/doc ./master
    echo "<meta http-equiv=refresh content=0;url=lettre/index.html>" > ./master/index.html
elif [ "$TRAVIS_TAG" != ""  ]; then
    rm -rf $TRAVIS_TAG
    mv ../target/doc ./$TRAVIS_TAG
    echo "<meta http-equiv=refresh content=0;url=lettre/index.html>" > ./$TRAVIS_TAG/index.html

    latest=$(echo * | tr " " "\n" | sort -V -r | head -n1)
    if [ "$TRAVIS_TAG" == "$latest" ]; then

        echo "<meta http-equiv=refresh content=0;url=$latest/lettre/index.html>" > index.html
    fi
else
    exit 0
fi

git add -A .
git commit -m "Rebuild pages at ${TRAVIS_COMMIT}"
git push --quiet origin gh-pages
