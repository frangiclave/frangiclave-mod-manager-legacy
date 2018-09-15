#!/usr/bin/env bash

set -e

# Get the current operating system
if [ "$(uname)" == "Darwin" ]
then
    brew update
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]
then
    sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF
    echo "deb https://download.mono-project.com/repo/ubuntu stable-trusty main" | sudo tee /etc/apt/sources.list.d/mono-official-stable.list
    sudo apt-get update
fi

set +e

