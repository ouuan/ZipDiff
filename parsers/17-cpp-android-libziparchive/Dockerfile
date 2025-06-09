FROM debian:12-slim AS build

RUN echo 'deb http://deb.debian.org/debian bookworm-backports main' > /etc/apt/sources.list.d/backports.list
RUN apt-get update
RUN apt-get install -y g++
RUN apt-get install -t bookworm-backports -y android-libziparchive-dev android-libbase-dev

WORKDIR /src
COPY unzip.cpp .
RUN g++ unzip.cpp -O2 -o unzip -I/usr/include/android -L/usr/lib/x86_64-linux-gnu/android -lziparchive

FROM debian:12-slim

COPY --from=build /usr/lib/x86_64-linux-gnu/android/*.so.0 /usr/lib/
COPY --from=build /src/unzip /
COPY unzip-all.sh /
ENTRYPOINT ["/unzip-all.sh"]
