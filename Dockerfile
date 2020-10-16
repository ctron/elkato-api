FROM docker.io/library/rust:latest as builder

RUN mkdir /src
ADD . /src

WORKDIR /src

RUN cargo build --release

WORKDIR /

FROM registry.access.redhat.com/ubi8/ubi-minimal:latest

COPY --from=builder /src/target/release/elkato-proxy /elkato-proxy

CMD /elkato-proxy
EXPOSE 8080
