# docker image build --tag egoroff/egoroff .

# build UI
FROM oven/bun:1 AS node-build
WORKDIR /app
COPY ui/package.json .
COPY ui/public/ ./public/
RUN ls -lah ./public
RUN bun install
COPY ui/src/ ./src/
COPY static/img/ /static/img/
COPY static/map.json /static/
COPY static/config.json /static/
COPY static/robots.txt /static/
COPY ui/vue.config.js ./
COPY ui/tsconfig.json ./
COPY ui/bun.lock ./
RUN bun run build

# Build service
FROM rust:alpine AS rust-build
WORKDIR /egoroff
RUN apk add musl-dev lld openssl-dev ca-certificates curl && update-ca-certificates
COPY --from=node-build /static /static
COPY apache/ /apache/
COPY templates/apache/ /templates/apache/
COPY egoroff/.cargo/ ./.cargo/
COPY egoroff/kernel/ ./kernel/
COPY egoroff/migrate/ ./migrate/
COPY egoroff/server/ ./server/
COPY egoroff/egoroff/ ./egoroff/
COPY egoroff/Cargo.toml ./
COPY egoroff/Cargo.lock ./
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build -p egoroff --release --target x86_64-unknown-linux-musl --locked

FROM gcr.io/distroless/static-debian13:latest
ENV EGOROFF_HTTP_PORT=4200 \
    EGOROFF_HTTPS_PORT=4201 \
    EGOROFF_CERT_DIR=/data/certs \
    EGOROFF_DATA_DIR=/data/data \
    EGOROFF_HOME_DIR=/
COPY --from=rust-build /apache/config.json /apache/
COPY --from=node-build /static /static
COPY --from=rust-build /egoroff/target/x86_64-unknown-linux-musl/release/egoroff /usr/local/bin/egoroff
ENTRYPOINT [ "/usr/local/bin/egoroff" ]
CMD [ "server" ]
EXPOSE 4200
EXPOSE 4201
