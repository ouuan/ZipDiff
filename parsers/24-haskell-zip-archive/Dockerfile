FROM haskell:9-slim AS build

RUN cabal update
RUN cabal install zip-archive-0.4.3.2 -f executable

FROM debian:12-slim

COPY --from=build /root/.local/bin/zip-archive /

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
