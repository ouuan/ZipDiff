FROM debian:12-slim AS build

RUN apt-get update
RUN apt-get install -y gcc

ADD https://github.com/kuba--/zip/archive/refs/tags/v0.3.2.tar.gz src.tar.gz
RUN tar xf src.tar.gz && mv zip-*/src .

WORKDIR /src

COPY unzip.c .

RUN gcc unzip.c zip.c -O2 -o unzip

FROM debian:12-slim

COPY --from=build /src/unzip /

COPY unzip-all.sh /
ENTRYPOINT ["/unzip-all.sh"]
