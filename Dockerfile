FROM rust:1.53.0 as build
WORKDIR /slack-bot
COPY . .
RUN cargo build --release

FROM alpine:latest
WORKDIR /slack-music-bot
COPY --from=build /slack-bot/target/release/main .
EXPOSE 5000
CMD ["./main"]

