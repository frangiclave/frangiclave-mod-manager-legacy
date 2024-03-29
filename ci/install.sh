#!/usr/bin/env bash

set -e

# Get the current operating system
if [ "$(uname)" == "Darwin" ]
then
    brew upgrade
    brew install git mono nuget openssl unzip
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]
then
    sudo apt-get -y install build-essential curl gcc git mono-devel musl-tools nuget unzip wget
    rustup target add x86_64-unknown-linux-musl
fi

rustup self update

set +e

