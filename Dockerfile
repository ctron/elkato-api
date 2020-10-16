FROM registry.access.redhat.com/ubi8

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH=$PATH:/root/.cargo/bin
RUN echo $PATH

RUN cargo build --release
