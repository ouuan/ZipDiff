FROM haskell:9-slim AS build

RUN apt-get update
RUN apt-get install -y libbz2-dev

RUN cabal update
RUN cabal install zip-2.1.0

FROM debian:12-slim

WORKDIR /data

RUN apt-get update
RUN apt-get install -y bzip2

COPY --from=build /root/.local/bin/haskell-zip-app /

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
