FROM debian
RUN apt-get update; apt-get install -y cmake libasound2-dev libudev-dev libatk1.0-dev libgtk-3-dev curl g++
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install nightly
RUN rustup component add clippy
