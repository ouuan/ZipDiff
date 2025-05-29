FROM freepascal/fpc:3.2.2-full AS build

WORKDIR /src

COPY unzip.pp .

RUN fpc unzip

FROM alpine:3.20

COPY --from=build /src/unzip /

COPY unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
