FROM rust:1.80-alpine as builder
ENV NAME=rust-docker

# First build a dummy project with our dependencies to cache them in Docker
WORKDIR /usr/src
#RUN cargo new --bin ${NAME}
WORKDIR /usr/src/${NAME}
#COPY ./Cargo.lock ./Cargo.lock
#COPY ./Cargo.toml ./Cargo.toml
#RUN cargo build --release
#RUN ls -lah /usr/src/${NAME}/target/release/
#RUN rm src/*.rs
RUN apk add --no-cache musl-dev

# Now copy the sources and do the real build
COPY . .
#RUN cargo test
RUN cargo build --release 

# Second stage putting the build result into a debian jessie-slim image
FROM debian:bullseye
ENV NAME=rust-docker
RUN apt update 
#RUN apt-get install -y libssl-dev

COPY --from=builder /usr/src/${NAME}/target/release/${NAME} /usr/local/bin/${NAME}
COPY --from=builder /usr/src/${NAME}/config.json .
CMD ${NAME}