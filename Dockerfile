# docker image build --tag egoroff/egoroff .

# build UI
FROM node:lts AS node-build
WORKDIR /app
COPY ui/package.json .
RUN npm i -f
COPY ui/src/ ./src/
COPY ui/public/ ./public/
COPY static/img/ /static/img/
COPY static/map.json /static/
COPY static/robots.txt /static/
COPY ui/vue.config.js ./
COPY ui/tsconfig.json ./
COPY ui/babel.config.js ./
RUN npm run build

# Build service
FROM rust:latest as rust-build
WORKDIR /egoroff
COPY --from=node-build /static /static
COPY egoroff/kernel/ ./kernel/
COPY egoroff/server/ ./server/
COPY egoroff/egoroff/ ./egoroff/
COPY egoroff/Cargo.toml ./
RUN cargo test --workspace --all-features --release
RUN cargo build --workspace --release

FROM gcr.io/distroless/cc-debian11:latest
#FROM debian:11-slim
ENV EGOROFF_HTTP_PORT=4200
ENV EGOROFF_HTTPS_PORT=4201
ENV EGOROFF_CERT_DIR=/data/certs
ENV EGOROFF_HOME_DIR=/
COPY --from=rust-build /static /static
COPY --from=rust-build /egoroff/target/release/egoroff /usr/local/bin/egoroff
USER root
ENTRYPOINT [ "/usr/local/bin/egoroff" ]
CMD [ "server" ]
EXPOSE 4200
EXPOSE 4201