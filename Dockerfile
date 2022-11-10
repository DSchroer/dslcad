FROM debian
RUN apt-get update; apt-get install -y cmake libasound2-dev libudev-dev libatk1.0-dev libgtk-3-dev curl g++ mingw-w64 zip
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install nightly
RUN rustup toolchain install nightly-x86_64-pc-windows-gnu
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup component add clippy
