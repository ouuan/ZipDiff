FROM node:20-alpine

WORKDIR /src

RUN apk add parallel

RUN npm init -y

RUN npm install adm-zip@0.5.16

COPY unzip.js .

COPY unzip /

COPY testcase.sh parallel-unzip-all.sh /
ENTRYPOINT ["/parallel-unzip-all.sh"]
