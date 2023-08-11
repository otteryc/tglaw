FROM rust:latest
WORKDIR /usr/src/tglaw
COPY . .
COPY .env .

RUN cargo build --release
CMD source .env && /usr/src/tglaw/target/release/tglaw
