FROM gradle:8.10-jdk21-alpine AS build

COPY unzip /unzip

WORKDIR /unzip

RUN gradle build

FROM eclipse-temurin:21.0.5_11-jre-alpine

RUN apk add parallel

COPY --from=build /unzip/build/libs/unzip.jar /

COPY unzip.sh /unzip

COPY testcase.sh parallel-unzip-all.sh /
ENTRYPOINT ["/parallel-unzip-all.sh"]
