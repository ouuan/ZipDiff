FROM alpine:3.20

RUN apk add php83
RUN apk add php83-zip

COPY unzip.php /

COPY unzip unzip-all.sh /

ENTRYPOINT [ "/unzip-all.sh" ]
