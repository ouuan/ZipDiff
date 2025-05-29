FROM debian:12-slim AS build

RUN apt-get update
RUN apt-get install -y gcc make

WORKDIR /tmp
ADD https://github.com/madler/zlib/releases/download/v1.3.1/zlib-1.3.1.tar.gz .
RUN tar xf zlib-*.tar.gz
RUN mv zlib-*/ /src

WORKDIR /src
RUN ./configure
RUN make libz.a
RUN make -C contrib/minizip miniunz

FROM debian:12-slim

COPY --from=build /src/contrib/minizip/miniunz /usr/local/bin/miniunzip

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
