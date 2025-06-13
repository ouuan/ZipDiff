FROM alpine:3.20

RUN apk add minizip-ng --repository=https://dl-cdn.alpinelinux.org/alpine/edge/community --repository=https://dl-cdn.alpinelinux.org/alpine/edge/main

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
