FROM alpine:3.20

RUN apk add minizip-ng --repository=https://dl-cdn.alpinelinux.org/alpine/edge/community

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
