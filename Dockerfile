FROM lukemathwalker/cargo-chef:latest-rust-1.72.0
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
# Compute a lock-like file for our project
RUN bash
