FROM debian:12-slim AS build

RUN apt-get update
RUN apt-get install -y gprbuild gnat-12 unzip

ADD https://github.com/zertovitch/zip-ada/archive/aaba1a767a47851df075a9884457052719e0488f.zip src.zip
RUN unzip src.zip
RUN mv zip-ada-* src

WORKDIR /src
RUN gprbuild -p -P zipada.gpr

FROM debian:12-slim

RUN apt-get update
RUN apt-get install -y libgnat-12

COPY --from=build /src/unzipada /

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
