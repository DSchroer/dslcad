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
    wget \
    libxml2-dev \
    libssl-dev \
    gnupg2 \
    apt-transport-https  \
    ca-certificates  \
    software-properties-common

RUN curl -fsSL https://download.docker.com/linux/debian/gpg | apt-key add -
RUN add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/debian $(lsb_release -cs) stable"
RUN apt-get update; apt-get install -y docker-ce

# OSXCross for Mac Builds
RUN git clone https://github.com/tpoechtrager/osxcross
RUN cd osxcross && \
    wget -nc https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz && \
    mv MacOSX11.3.sdk.tar.xz tarballs/ && \
    UNATTENDED=yes ./build.sh
RUN ln -s /osxcross/target/bin/x86_64-apple-darwin20.4-ld /osxcross/target/bin/x86_64-apple-darwin-ld

# Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Cross
RUN cargo install cross --git https://github.com/cross-rs/cross
ENV CROSS_CONTAINER_IN_CONTAINER=true

# Toolchains
RUN rustup toolchain install nightly
RUN rustup target add x86_64-apple-darwin
