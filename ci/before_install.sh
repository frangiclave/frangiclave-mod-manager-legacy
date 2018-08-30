#!/usr/bin/env bash

set -e

if [ $TRAVIS_OS_NAME = linux ]
then
    sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF
    echo "deb https://download.mono-project.com/repo/ubuntu stable-trusty main" | sudo tee /etc/apt/sources.list.d/mono-official-stable.list
    sudo apt-get -qq update
else
    brew update
fi

set +e
