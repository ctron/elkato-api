FROM registry.access.redhat.com/ubi8

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN cargo build --release
