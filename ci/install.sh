#!/usr/bin/env bash

set -e

if [ $TRAVIS_OS_NAME = linux ]
then
    sudo apt-get install -qq build-essential curl gcc git libssl-dev mono-devel nuget unzip wget
else
    brew install git mono nuget openssl unzip
fi

rustup self update

set +e
