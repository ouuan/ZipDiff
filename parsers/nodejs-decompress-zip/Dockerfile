FROM node:20-alpine

WORKDIR /src

RUN apk add parallel

RUN npm init -y

RUN npm install decompress-zip@0.3.3

COPY unzip.js .

COPY unzip testcase.sh parallel-unzip-all.sh /

ENTRYPOINT ["/parallel-unzip-all.sh", "50%"]
