FROM alpine:3.20 AS build

RUN apk add g++ make cmake

ADD https://github.com/pocoproject/poco/archive/refs/tags/poco-1.13.3-release.tar.gz src.tar.gz
RUN tar xf src.tar.gz && mv poco-* poco

WORKDIR /poco
RUN cmake -S . -B build -D BUILD_SHARED_LIBS=OFF -D ENABLE_ZIP=ON -D ENABLE_FOUNDATION=ON -D ENABLE_XML=ON -D ENABLE_ENCODINGS=OFF -D ENABLE_ENCODINGS_COMPILER=OFF -D ENABLE_JSON=OFF -D ENABLE_MONGODB=OFF -D ENABLE_DATA_SQLITE=OFF -D ENABLE_REDIS=OFF -D ENABLE_PROMETHEUS=OFF -D ENABLE_UTIL=OFF -D ENABLE_NET=OFF -D ENABLE_SEVENZIP=OFF -D ENABLE_CPPPARSER=OFF -D ENABLE_POCODOC=OFF -D ENABLE_PAGECOMPILER=OFF -D ENABLE_PAGECOMPILER_FILE2PAGE=OFF -D ENABLE_ACTIVERECORD=OFF -D ENABLE_ACTIVERECORD_COMPILER=OFF
RUN cmake --build build --config Release -j$(nproc)
RUN cmake --install build

WORKDIR /src
COPY unzip.cpp .
RUN g++ unzip.cpp -O2 -o unzip -static -lPocoZip -lPocoFoundation

FROM alpine:3.20
COPY --from=build /src/unzip /
COPY unzip-all.sh /
ENTRYPOINT ["/unzip-all.sh"]
