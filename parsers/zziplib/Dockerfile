FROM debian:12-slim AS build

RUN apt-get update
RUN apt-get install -y cmake zlib1g-dev

WORKDIR /tmp
ADD https://github.com/gdraheim/zziplib/archive/refs/tags/v0.13.78.tar.gz .
RUN tar xf *.tar.gz
RUN mv zziplib-*/ /src

WORKDIR /src
RUN cmake -S . -B build -D ZZIPBINS=On -D BUILD_SHARED_LIBS=Off -D ZZIPWRAP=Off -D ZZIPSDL=Off -D ZZIPTEST=Off -D ZZIPDOCS=Off
RUN cmake --build build

FROM debian:12-slim

RUN apt-get update
RUN apt-get install -y zlib1g

COPY --from=build /src/build/bins/unzzip /usr/local/bin

COPY unzip unzip-all.sh /

ENTRYPOINT [ "/unzip-all.sh" ]
