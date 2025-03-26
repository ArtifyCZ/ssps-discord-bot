FROM rust:1.84 AS builder

WORKDIR /app

COPY Cargo.lock Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs

COPY resources ./resources
COPY src ./src
RUN cargo install --path .

FROM debian:bookworm

WORKDIR /app

ARG UNAME=ssps-discord-bot
ARG UGROUP=ssps-discord-bot
ARG UID=541
ARG GID=541

RUN groupadd -g $GID $UGROUP \
    && useradd -u $UID -g $GID -s /bin/bash $UNAME \
    && chown -R $UNAME:$UGROUP /app \
    && chmod -R 755 /app \
    && apt-get update \
    && apt-get install -y libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder --chown=$UNAME:$UGROUP --chmod=755 /usr/local/cargo/bin/ssps-discord-bot /usr/local/bin/ssps-discord-bot

USER $UNAME

CMD ["ssps-discord-bot"]
