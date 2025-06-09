FROM node:20-alpine

RUN npm install extract-zip@2.0.1 -g

WORKDIR /usr/local/lib/node_modules/extract-zip
RUN sed -i 's/"yauzl":.*/"yauzl": "3.2.0"/' package.json
RUN npm install

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
