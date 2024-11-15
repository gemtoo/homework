FROM rust:1.80-bookworm

WORKDIR /app
COPY . /app
RUN cargo install --path /app

ENTRYPOINT program-manager
