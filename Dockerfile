FROM alpine:latest

WORKDIR /app

COPY . .

RUN apk update && apk add --no-cache make curl musl-dev gcc nasm grub xorriso 

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain nightly \
    --profile minimal \
    --no-modify-path

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-musl && \
    wget http://ftp.debian.org/debian/pool/main/g/grub2/grub-pc-bin_2.12~rc1-12_amd64.deb && \
    ar x grub-pc-bin_2.12\~rc1-12_amd64.deb && tar -xf data.tar.xz -C

CMD ["/bin/sh"]