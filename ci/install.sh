#!/usr/bin/env bash

set -e

source ~/.cargo/env || true

if [ $TRAVIS_OS_NAME = linux ]
then
    sudo apt-get -yqq curl install build-essential gcc git libssl-dev mono-devel nuget unzip wget
else
    brew install mono
fi

rustup self update

set +e
