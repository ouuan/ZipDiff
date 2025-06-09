FROM swift:5.10 AS build

COPY src /src
WORKDIR /src

RUN swift build -c release --static-swift-stdlib
RUN mv "$(swift build -c release --show-bin-path)/unzip" /

FROM debian:12-slim

RUN apt-get update
RUN apt-get install -y parallel

COPY --from=build /unzip /

COPY parallel-unzip-all.sh testcase.sh /

ENTRYPOINT [ "/parallel-unzip-all.sh" ]
