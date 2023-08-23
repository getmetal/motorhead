FROM rust:1.68-bullseye as build

# had to add this for open-ssl
RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev ca-certificates && \
  rustup target add x86_64-unknown-linux-gnu

RUN USER=root cargo new --bin motorhead
WORKDIR /motorhead

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/motorhead*
RUN cargo build --release

FROM debian:bullseye

RUN apt-get update && apt install -y openssl ca-certificates

COPY --from=build /motorhead/target/release/motorhead .

CMD ["./motorhead"]
