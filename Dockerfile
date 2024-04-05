FROM rust:1.77 as builder
WORKDIR /usr/src/tg-captain
COPY . .
RUN cargo install --path .

FROM debian:bookworm
COPY --from=builder /usr/local/cargo/bin/tg-captain /usr/local/bin/tg-captain
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates
RUN mkdir /data
RUN mkdir /data/config
ENV CONFIG_PATH=/data/config/config.yml
CMD ["tg-captain"]