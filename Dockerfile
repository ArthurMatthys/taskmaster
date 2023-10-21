FROM rust:buster 
WORKDIR /app
RUN apt update && apt install lld clang vim -y
COPY . .
# Compute a lock-like file for our project
Run cargo build
RUN bash
