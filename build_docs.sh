#!/usr/bin/env bash

cargo doc &&
dir=`mktemp -d 2>/dev/null || mktemp -d -t 'mytmpdir'` &&
cp -r ./target/doc/ $dir/ &&
git checkout gh-pages &&
cp -r $dir/* .
