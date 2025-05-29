FROM rust:1.82-slim-bookworm AS build

WORKDIR /src/unzip

RUN cargo init

RUN cargo add zip@2.2.0

COPY unzip.rs src/main.rs

RUN cargo build --release

FROM debian:12-slim

COPY --from=build /src/unzip/target/release/unzip /

COPY unzip-all.sh /

ENTRYPOINT [ "/unzip-all.sh" ]
