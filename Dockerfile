FROM registry.access.redhat.com/ubi8

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH=$PATH:$HOME/.cargo/bin

RUN cargo build --release
