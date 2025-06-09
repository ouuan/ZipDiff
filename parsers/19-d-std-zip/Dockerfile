FROM alpine:3.20 AS build

WORKDIR /src

RUN apk add dmd gcc musl-dev --repository=https://dl-cdn.alpinelinux.org/alpine/edge/community

COPY unzip.d .

RUN dmd unzip.d

FROM alpine:3.20

RUN apk add llvm-libunwind

COPY --from=build /src/unzip /

COPY unzip-all.sh /
ENTRYPOINT ["/unzip-all.sh"]
