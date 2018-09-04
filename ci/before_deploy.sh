#!/usr/bin/env bash

set -e

# Try to find the source used for this deploy
if [ -z ${TRAVIS_BUILD_DIR+x} ]
then
    export MM_DIR=$(PWD)
else
    export MM_DIR=${TRAVIS_BUILD_DIR}
fi
export MM_PATCH_DIR=${MM_DIR}/data/patch

# Get the current operating system
if [ "$(uname)" == "Darwin" ]
then
    export OS="macos"
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]
then
    export OS="linux"
else
    export OS="unknown"
fi

export BUILD_DIR=/tmp/frangiclave-build

export MONOMOD_DIR=${BUILD_DIR}/MonoMod
export MONOMOD_BIN_DIR=${MONOMOD_DIR}/MonoMod/bin/Release

export PATCH_DIR=${BUILD_DIR}/frangiclave-patch
export PATCH_MONOMOD_DIR=${PATCH_DIR}/MonoMod
export PATCH_CS_DIR=${PATCH_DIR}/CultistSimulator
export PATCH_BIN_DIR=${PATCH_DIR}/Assembly-CSharp/bin/Release

export ARTIFACT_DIR=${BUILD_DIR}/artifacts

# Prepare the build directory
echo "Preparing build directory"
rm -rf ${BUILD_DIR}
mkdir ${BUILD_DIR}
mkdir ${ARTIFACT_DIR}
cd ${BUILD_DIR}

# Fetch sources
echo "Fetching MonoMod"
git clone -q https://github.com/0x0ade/MonoMod
echo "Fetching frangiclave-patch"
git clone -q https://gitlab.com/frangiclave/frangiclave-patch

# Build MonoMod and bundle Mono together with it for easier distribution
echo "Building MonoMod"
cd ${MONOMOD_DIR}
nuget restore -NonInteractive -Verbosity quiet
msbuild /p:Configuration=Release /clp:ErrorsOnly
cd ${MONOMOD_BIN_DIR}
mkbundle --simple MonoMod.exe --no-machine-config --no-config -o MonoMod_bundled.exe

# Get Cultist Simulator DLLs
echo "Fetching Cultist Simulator DLLs"
cd ${PATCH_CS_DIR}
wget -q ${CS_DLLS_URL}
unzip -qq Assembly-CSharp.zip
rm Assembly-CSharp.zip

# Build the patch
echo "Building Frangiclave Patch"
cd ${PATCH_DIR}
cp ${MONOMOD_BIN_DIR}/*.dll ${PATCH_MONOMOD_DIR}
cp ${MONOMOD_BIN_DIR}/MonoMod.exe ${PATCH_MONOMOD_DIR}
msbuild /p:Configuration=Release /clp:ErrorsOnly

# Build the Mod Manager, copying the version of MonoMod with Mono bundled for
# portability
echo "Building Frangiclave Mod Manager"
cd ${MM_DIR}
cp ${PATCH_BIN_DIR}/*.dll ${MM_PATCH_DIR}
cp ${MONOMOD_BIN_DIR}/MonoMod_bundled.exe ${MM_PATCH_DIR}/MonoMod.exe
if [ ${OS} == "linux" ]
then
    cargo build -q --release --target x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-musl/release/frangiclave-mod-manager ${ARTIFACT_DIR}/frangiclave-mod-manager-${OS}
else
    cargo build -q --release
    cp target/release/frangiclave-mod-manager ${ARTIFACT_DIR}/frangiclave-mod-manager-${OS}
fi

echo "Complete"

set +e
