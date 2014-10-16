#!/usr/bin/env bash

cargo doc &&
dir=`mktemp -d 2>/dev/null || mktemp -d -t 'mytmpdir'` &&
cp -r ./doc/ $dir/doc/ &&
git checkout gh-pages &&
cp -r $dir/doc/* .
