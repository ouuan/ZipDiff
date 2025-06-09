FROM alpine:3.20

RUN apk add perl-archive-zip

COPY unzip.pl /

COPY unzip unzip-all.sh /

ENTRYPOINT [ "/unzip-all.sh" ]
