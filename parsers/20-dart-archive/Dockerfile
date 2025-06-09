FROM dart:3.5 AS build

WORKDIR /src

COPY pubspec.yaml .
RUN dart pub get

COPY unzip.dart .
RUN dart compile exe unzip.dart -o unzip

FROM debian:12-slim
COPY --from=build /src/unzip /
COPY unzip-all.sh /
ENTRYPOINT ["/unzip-all.sh"]
