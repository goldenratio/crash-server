# Build stage
FROM rust:1.80-alpine3.19 AS builder
LABEL MAINTAINER="goldenratio"

# This is important, see https://github.com/rust-lang/docker-rust/issues/85
ENV RUSTFLAGS="-C target-feature=-crt-static"

# if needed, add additional dependencies here
RUN apk add --no-cache musl-dev

WORKDIR /crash-server
COPY . .

RUN cargo build --release

############################################################
# Final Image
FROM alpine:3.19

# if needed, install additional dependencies here
RUN apk add --no-cache libgcc

WORKDIR /crash-server

COPY --from=builder /crash-server/target/release/crash-server .

ENV RUST_LOG=${RUST_LOG:-info}
ENV RUST_BACKTRACE=${RUST_BACKTRACE:-full}
ENV PORT=${PORT:-8090}

EXPOSE $PORT

ENTRYPOINT ["./crash-server"]
