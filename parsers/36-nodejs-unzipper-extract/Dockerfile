FROM node:20-alpine

WORKDIR /src

RUN npm init -y

RUN npm install unzipper@0.12.3

COPY unzip.js .

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
