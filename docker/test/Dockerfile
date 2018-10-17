FROM ekidd/rust-musl-builder:nightly AS builder
ADD . ./
RUN sudo chown -R rust:rust /home/rust
RUN cargo build --release

FROM alpine:latest
RUN mkdir -p /opt/deadmock/bin
RUN mkdir -p /root/.config/deadmock
RUN apk add --no-cache ca-certificates
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/deadmock \
    /opt/deadmock/bin/
EXPOSE 32276
CMD ["/opt/deadmock/bin/deadmock", "-vvv"]