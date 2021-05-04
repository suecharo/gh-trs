FROM rust:1.51.0-slim-buster

RUN apt update && \
    apt install -y --no-install-recommends \
    git \
    tini && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

ENTRYPOINT ["tini", "--"]
CMD ["sleep", "infinity"]
