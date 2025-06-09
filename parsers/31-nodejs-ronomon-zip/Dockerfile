FROM node:20-alpine

WORKDIR /src

RUN npm init -y

RUN npm install @ronomon/zip@1.12.0

COPY unzip.js .

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
