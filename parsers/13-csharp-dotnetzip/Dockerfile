FROM mcr.microsoft.com/dotnet/sdk:9.0-alpine AS build

WORKDIR /unzip

RUN dotnet new console

RUN dotnet add package DotNetZip --version 1.16.0

RUN rm ./*.cs

COPY unzip.cs .

RUN dotnet build -c Release

FROM mcr.microsoft.com/dotnet/runtime:9.0-alpine

RUN apk add parallel

COPY --from=build /unzip/bin/Release/net9.0 /build

RUN ln -s /build/unzip /unzip

COPY testcase.sh parallel-unzip-all.sh /
ENTRYPOINT ["/parallel-unzip-all.sh"]
