FROM golang:1.22.3-alpine3.20 AS build

WORKDIR /src

RUN go mod init main

RUN go get github.com/evilsocket/islazy/zip

COPY unzip.go .

RUN go build unzip.go

FROM alpine:3.20

COPY --from=build /src/unzip /

COPY unzip-all.sh /
ENTRYPOINT ["/unzip-all.sh"]
