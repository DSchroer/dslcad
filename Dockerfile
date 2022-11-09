FROM rust
RUN apt update; apt install -y cmake libasound2-dev libudev-dev libatk1.0-dev libgtk-3-dev
RUN rustup component add clippy
