FROM golang:1.22.3-alpine3.20 AS build

RUN apk add gcc musl-dev

RUN go install github.com/gen2brain/go-unarr/cmd/unarr@v0.2.4

FROM alpine:3.20

COPY --from=build /go/bin/unarr /

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
