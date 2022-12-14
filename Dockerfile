# docker image build --tag egoroff/egoroff .

# build UI
FROM node:lts AS node-build
WORKDIR /app
COPY ui/package.json .
COPY ui/public/ ./public/
RUN find ./public/**/*_r.html | sed -r -e 's/((.+)_r.html)/\1 \2.html/g' | xargs -I % bash -c 'mv -v -f %'
RUN find ./public/*_r.html | sed -r -e 's/((.+)_r.html)/\1 \2.html/g' | xargs -I % bash -c 'mv -v -f %'
RUN ls -lah ./public
RUN npm i -f
COPY ui/src/ ./src/
COPY static/img/ /static/img/
COPY static/map.json /static/
COPY static/config.json /static/
COPY static/robots.txt /static/
COPY ui/vue.config.js ./
COPY ui/tsconfig.json ./
COPY ui/babel.config.js ./
RUN npm run build

# Build service
FROM rust:latest as rust-build
WORKDIR /egoroff
COPY --from=node-build /static /static
RUN ls -lah /static
COPY apache/ /apache/
RUN ls -lah /apache
COPY templates/apache/ /templates/apache/
RUN ls -lah /templates/apache
COPY egoroff/kernel/ ./kernel/
COPY egoroff/migrate/ ./migrate/
COPY egoroff/server/ ./server/
COPY egoroff/egoroff/ ./egoroff/
COPY egoroff/Cargo.toml ./
RUN cargo test --workspace --release
RUN cargo build --workspace --all-features --release

FROM gcr.io/distroless/cc-debian11:latest
ENV EGOROFF_HTTP_PORT=4200
ENV EGOROFF_HTTPS_PORT=4201
ENV EGOROFF_CERT_DIR=/data/certs
ENV EGOROFF_DATA_DIR=/data/data
ENV EGOROFF_HOME_DIR=/
COPY --from=rust-build /apache/config.json /apache/
COPY --from=rust-build /static /static
COPY --from=rust-build /egoroff/target/release/egoroff /usr/local/bin/egoroff
USER root
ENTRYPOINT [ "/usr/local/bin/egoroff" ]
CMD [ "server" ]
EXPOSE 4200
EXPOSE 4201
