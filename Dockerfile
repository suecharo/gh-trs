FROM debian:stretch-slim

LABEL org.opencontainers.image.authors="suecharo <suehiro619@gmail.com>"
LABEL org.opencontainers.image.url="https://github.com/suecharo/gh-trs"
LABEL org.opencontainers.image.source="https://github.com/suecharo/gh-trs/blob/main/Dockerfile"
LABEL org.opencontainers.image.version="1.1.11"
LABEL org.opencontainers.image.description="CLI tool to publish and test your own GA4GH TRS API using GitHub"
LABEL org.opencontainers.image.licenses="Apache2.0"

ADD https://github.com/suecharo/gh-trs/releases/latest/download/gh-trs /usr/bin/
RUN chmod +x /usr/bin/gh-trs

WORKDIR /app

ENTRYPOINT [""]
CMD ["sleep", "infinity"]