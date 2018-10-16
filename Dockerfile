FROM ekidd/rust-musl-builder:nightly AS builder
ADD . ./
RUN sudo chown -R rust:rust /home/rust
RUN cargo build

FROM alpine:latest
RUN mkdir -p /opt/deadmock/bin
RUN mkdir -p /opt/deadmock/config/deadmock
RUN apk add --no-cache ca-certificates
COPY --from=builder \
    /home/rust/src/config/* \
    /opt/deadmock/config/deadmock/
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/debug/deadmock \
    /opt/deadmock/bin/
CMD ["/opt/deadmock/bin/deadmock", "-v"]