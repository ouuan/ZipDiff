FROM debian:12-slim

RUN echo 'deb http://deb.debian.org/debian bookworm-backports main' > /etc/apt/sources.list.d/backports.list
RUN apt-get update
RUN apt-get install -t bookworm-backports -y 7zip

COPY unzip unzip-all.sh /

ENTRYPOINT ["/unzip-all.sh"]
