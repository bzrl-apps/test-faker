FROM debian:bullseye-slim AS builder

RUN mkdir -p /app
WORKDIR /app

COPY test-faker-*.tar.gz /app
RUN ls -la && pwd
RUN tar -xzf test-faker-*-$(dpkg --print-architecture)-linux.tar.gz

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates tzdata systemd && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app/plugins

COPY --from=builder /app/test-faker /app/test-faker
COPY --from=builder /app/*.so /app/plugins/

WORKDIR /app
VOLUME /app/flows/

#RUN ./test-faker server
