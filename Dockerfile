FROM debian
RUN apt-get update; apt-get install -y \
    clang \
    cmake \
    libasound2-dev \
    libudev-dev \
    libatk1.0-dev \
    libgtk-3-dev \
    curl \
    gcc \
    g++ \
    mingw-w64 \
    zip \
    zlib1g-dev \
    libmpc-dev \
    libmpfr-dev \
    libgmp-dev \
    git \
    wget


RUN git clone https://github.com/tpoechtrager/osxcross

RUN apt-get install -y libxml2-dev libssl-dev

RUN cd osxcross && \
    wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.10.sdk.tar.xz && \
    mv MacOSX10.10.sdk.tar.xz tarballs/ && \
    UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup toolchain install nightly
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-apple-darwin
RUN rustup component add clippy
