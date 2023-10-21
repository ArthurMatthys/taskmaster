# README

For a good experience, run these tests/commands from the root of the `tests` folder.

You should see (not updated):

```shell
tree
.
├── README.md
├── binary
│   ├── go.mod
│   └── main.go
└── success
    └── config.yml
```

## 1. Compile the tester binary

This tester binary is a program loaded in the container. It accepts parameters impacting its behavior.
It is a perfect candidate to test a job controller.

```shell
GOOS=linux GOARCH=amd64 go build -C ./binary
```

> replace GOARCH your computer's platform

## 2. Run the containers

```shell
docker build -t task .
docker run --label task -it task bash
```

## 3. Connect to existing container (from other terminal)

```shell
docker exec -ti $(docker ps --filter label=task -q) bash
```
