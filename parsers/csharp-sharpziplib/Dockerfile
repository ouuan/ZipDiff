FROM mcr.microsoft.com/dotnet/sdk:9.0-alpine AS build

WORKDIR /unzip

RUN dotnet new console

RUN dotnet add package SharpZipLib --version 1.4.2

RUN rm ./*.cs

COPY unzip.cs .

RUN dotnet build -c Release

FROM mcr.microsoft.com/dotnet/runtime:9.0-alpine

COPY --from=build /unzip/bin/Release/net9.0 /build

RUN ln -s /build/unzip /unzip

COPY unzip-all.sh /
ENTRYPOINT ["/unzip-all.sh"]
