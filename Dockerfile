FROM rust:buster 
WORKDIR /app
RUN apt update && apt install lld clang vim -y

COPY . .


ENV TASKMASTER_LOGFILE="/app/taskmaster.log"
ENV SERVER_ADDRESS="localhost:4242"
ENV TASKMASTER_CONFIG_FILE_PATH="/app/tests/success/config.yml"

RUN cargo build
RUN bash
