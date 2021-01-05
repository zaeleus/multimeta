FROM rust:1.49.0-buster as env

RUN apt-get update \
      && apt-get --yes install --no-install-recommends \
        cmake \
        nasm \
      && rm -r /var/lib/apt/lists/*

RUN cd /tmp \
      && wget https://github.com/mozilla/mozjpeg/archive/v4.0.0.tar.gz \
      && echo "961e14e73d06a015e9b23b8af416f010187cc0bec95f6e3b0fcb28cc7e2cbdd4  v4.0.0.tar.gz" | sha256sum --check \
      && tar xf v4.0.0.tar.gz \
      && cd mozjpeg-4.0.0 \
      && mkdir build \
      && cd build \
      && cmake .. \
      && make --jobs $(nproc) install \
      && rm -r /tmp/mozjpeg-4.0.0

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
