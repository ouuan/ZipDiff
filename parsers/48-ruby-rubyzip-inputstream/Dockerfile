FROM ruby:3.3-alpine

RUN gem install rubyzip -v 2.3.2

COPY unzip.rb /

COPY unzip unzip-all.sh /

ENTRYPOINT [ "/unzip-all.sh" ]
