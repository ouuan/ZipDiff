FROM node:20-alpine

WORKDIR /src

RUN npm init -y

RUN npm install @zip.js/zip.js@2.7.53

COPY unzip.js .

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
