FROM rust:1.60 as builder

RUN USER=root cargo new --bin auth-app
WORKDIR ./auth-app
COPY ./Cargo.toml ./build.rs ./
RUN cargo build --release
RUN rm src/*.rs

ADD . ./


RUN cargo build --release

FROM debian:bullseye-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata curl \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 1500

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /auth-app/target/release/auth-app-rs ${APP}/auth-app

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./auth-app"]
