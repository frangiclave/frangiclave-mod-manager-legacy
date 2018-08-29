FROM ubuntu:latest
WORKDIR /frangiclave
RUN apt update && apt install curl && apt clean
RUN curl https://sh.rustup.rs -sSf --output rustup-init.sh && sh rustup-init.sh -y --default-toolchain none && export PATH=$HOME/.cargo/bin:$PATH
RUN rustup install stable-x86_64-pc-windows-msvc stable-x86_64-unknown-linux-gnu stable-x86_64-apple-darwin
RUN rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-msvc x86_64-apple-darwin
