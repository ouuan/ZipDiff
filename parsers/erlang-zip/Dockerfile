FROM erlang:27.1.2.0-alpine

RUN apk add parallel

COPY unzip parallel-unzip-all.sh testcase.sh /

ENTRYPOINT [ "/parallel-unzip-all.sh" ]
