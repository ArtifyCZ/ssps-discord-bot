FROM rust:1.80-bookworm AS builder

WORKDIR /app

COPY Cargo.lock Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm src/main.rs

COPY src ./src
RUN cargo build --release

FROM debian:bookworm

WORKDIR /app

ARG UNAME=ssps-discord-bot
ARG UGROUP=ssps-discord-bot
ARG UID=541
ARG GID=541

RUN groupadd -g $GID $UGROUP \
    && useradd -u $UID -g $GID -s /bin/bash $UNAME \
    && chown -R $UNAME:$UGROUP /app \
    && chmod -R 755 /app

COPY --from=builder --chown=$UNAME:$UGROUP /app/target/release/ssps-discord-bot .

USER $UNAME

CMD ["./ssps-discord-bot"]
