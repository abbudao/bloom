FROM rust:1.68-slim-bullseye AS build

WORKDIR /app
COPY . /app
RUN cargo clean && cargo build --release

FROM gcr.io/distroless/cc

WORKDIR /usr/src/bloom

COPY --from=build /app/target/release/bloom /usr/local/bin/bloom

CMD [ "bloom", "-c", "/etc/bloom.cfg" ]

EXPOSE 8080 8811
