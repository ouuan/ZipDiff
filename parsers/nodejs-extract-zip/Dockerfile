FROM node:20-alpine

RUN npm install extract-zip@2.0.1 -g

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
