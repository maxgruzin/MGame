FROM rust:1.93-bookworm AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release --no-default-features --bin party-server

FROM debian:bookworm-slim
RUN useradd --uid 10001 --create-home party
COPY --from=build /build/target/release/party-server /usr/local/bin/party-server
USER 10001:10001
WORKDIR /home/party
EXPOSE 7777/tcp
ENTRYPOINT ["party-server"]
