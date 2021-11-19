FROM rust:1.56.0 as build-env
WORKDIR /app
COPY . /app
ENV OPUS_STATIC=1
RUN cargo build --release

FROM archlinux
COPY --from=build-env /app/target/release/simple-music-bot /
RUN pacman -Syu --noconfirm youtube-dl ffmpeg
CMD ["./simple-music-bot"]
