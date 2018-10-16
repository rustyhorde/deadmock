FROM ekidd/rust-musl-builder:nightly AS builder
ADD . ./
RUN sudo chown -R rust:rust /home/rust
RUN cargo build

FROM alpine:latest
RUN mkdir -p /opt/deadmock/bin
RUN mkdir -p /root/.config/deadmock
RUN apk add --no-cache ca-certificates
ADD config.tar.xz \
    /root/.config/deadmock/
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/debug/deadmock \
    /opt/deadmock/bin/
RUN find /root/.config/deadmock -type f
EXPOSE 32276
CMD ["/opt/deadmock/bin/deadmock", "-vvv"]