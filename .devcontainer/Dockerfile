FROM ubuntu:24.04

USER root
ENV DEBIAN_FRONTEND=noninteractive
ENV TS=Etc/UTC
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

RUN apt-get update && apt-get install --no-install-recommends -qq -y \
    build-essential \
    clang \
    curl \
    git \
    gnupg \
    tzdata\
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    tpm2-tools \
    libtss2-dev \
    && apt-get -y -q upgrade \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable

RUN rustup component add clippy rustfmt

RUN cargo install --locked cargo-deny

# Intel SGX APT repository
RUN curl -fsSLo /usr/share/keyrings/intel-sgx-deb.asc https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key && \
    echo "deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-deb.asc] https://download.01.org/intel-sgx/sgx_repo/ubuntu noble main" \
    | tee /etc/apt/sources.list.d/intel-sgx.list

RUN apt-get update && apt-get install --no-install-recommends -qq -y libtdx-attest-dev \
    && apt-get -y -q upgrade \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
