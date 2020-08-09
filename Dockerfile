FROM rust:1.45.2-buster as env

RUN apt-get update \
      && apt-get --yes install --no-install-recommends \
        nasm \
      && rm -r /var/lib/apt/lists/*

RUN cd /tmp \
      && wget https://github.com/mozilla/mozjpeg/archive/v3.3.1.tar.gz \
      && echo "aebbea60ea038a84a2d1ed3de38fdbca34027e2e54ee2b7d08a97578be72599d  v3.3.1.tar.gz" | sha256sum --check \
      && tar xf v3.3.1.tar.gz \
      && cd mozjpeg-3.3.1 \
      && autoreconf -fvi \
      && ./configure --disable-dependency-tracking --with-jpeg8 \
      && make --jobs $(nproc) install \
      && rm -r /tmp/mozjpeg-3.3.1

COPY Cargo.lock Cargo.toml /tmp/multimeta/
COPY .git/ /tmp/multimeta/.git
COPY src/ /tmp/multimeta/src/

FROM env as builder

RUN cargo build --release --manifest-path /tmp/multimeta/Cargo.toml

FROM debian:buster

COPY --from=env /opt/mozjpeg/ /opt/mozjpeg/
COPY --from=builder /tmp/multimeta/target/release/multimeta /opt/multimeta/bin/

ENV MOZJPEG_HOME=/opt/mozjpeg

ENTRYPOINT ["/opt/multimeta/bin/multimeta"]
