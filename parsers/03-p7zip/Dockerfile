FROM debian:12-slim

RUN apt-get update
RUN apt-get install -y p7zip-full

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
