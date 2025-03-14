FROM ghcr.io/cosmian/intel-sgx:2.25

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable

RUN rustup component add clippy rustfmt

RUN apt-get update && apt-get install --no-install-recommends -qq -y \
    build-essential \
    pkg-config \
    libssl-dev \
    python3 \
    python3-pip \
    tpm2-tools \
    libtss2-dev \
    libtdx-attest-dev \
    && apt-get -y -q upgrade \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && python3 -m pip install "maturin"

RUN sed -i 's,https://localhost:8081/sgx/certification/v4/,https://pccs.staging.mse.cosmian.com/sgx/certification/v4/,' /etc/sgx_default_qcnl.conf
