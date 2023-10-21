FROM rust:buster 
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
# Compute a lock-like file for our project
RUN bash
