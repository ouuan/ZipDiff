FROM node:20-alpine

WORKDIR /src

RUN npm init -y

RUN npm install jszip@3.10.1

COPY unzip.js .

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
