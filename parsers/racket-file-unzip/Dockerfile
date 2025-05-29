FROM racket/racket:8.15-bc-full AS build

WORKDIR /src

COPY unzip.rkt .

RUN raco exe unzip.rkt

RUN raco distribute /unzip unzip

FROM debian:12-slim

RUN apt-get update
RUN apt-get install -y parallel

COPY --from=build /unzip /build

RUN ln -s /build/bin/unzip /unzip

COPY parallel-unzip-all.sh testcase.sh /

ENTRYPOINT [ "/parallel-unzip-all.sh" ]
