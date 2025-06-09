FROM alpine:3.20

WORKDIR /src

RUN apk add composer

RUN composer require nelexa/zip:4.0.2

COPY unzip.php .

COPY unzip unzip-all.sh /

ENTRYPOINT [ "/unzip-all.sh" ]
