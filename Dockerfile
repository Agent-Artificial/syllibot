FROM rust:latest

COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock
COPY src/ /app/src
COPY .env /app/.env

WORKDIR /app/

RUN cargo build --release

CMD ["./target/release/syllibot"]

