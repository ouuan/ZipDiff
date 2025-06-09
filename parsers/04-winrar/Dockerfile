FROM debian:12-slim

RUN dpkg --add-architecture i386
RUN sed -i 's/Components: main/Components: main contrib/g' /etc/apt/sources.list.d/debian.sources
RUN apt-get update
RUN apt-get install -y xvfb winetricks wine32 parallel

RUN winetricks msxml6 && sleep 1

ADD https://www.win-rar.com/fileadmin/winrar-versions/winrar/winrar-x32-701.exe /tmp/winrar.exe
RUN xvfb-run wine /tmp/winrar.exe /S && sleep 1

# abort build if installation fails
RUN stat '/root/.wine/drive_c/Program Files/WinRAR'

COPY run.sh parallel-unzip-all.sh testcase.sh unzip /

ENTRYPOINT ["/run.sh"]
