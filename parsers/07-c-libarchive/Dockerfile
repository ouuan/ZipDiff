FROM alpine:3.20 AS build

RUN apk add g++ make

WORKDIR /tmp
ADD https://github.com/libarchive/libarchive/releases/download/v3.7.7/libarchive-3.7.7.tar.gz .
RUN tar xf libarchive-*.tar.gz
RUN mv libarchive-*/ /src

WORKDIR /src
RUN ./configure
RUN make -j$(nproc --ignore=2) bsdunzip

FROM alpine:3.20

COPY --from=build /src/bsdunzip /usr/local/bin/

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
