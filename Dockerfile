FROM phusion/baseimage:0.11 as builder

ARG PROFILE=release
WORKDIR /node

COPY . /node

RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
        export PATH=$PATH:$HOME/.cargo/bin && \
        scripts/init.sh && \
        cargo build --$PROFILE

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11

ARG PROFILE=release

COPY --from=builder /node/target/$PROFILE/pki-node /usr/local/bin

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /node pki-node && \
	mkdir -p /node/.local/share/pki-node && \
	chown -R pki-node:pki-node /node/.local && \
	ln -s /node/.local/share/pki-node /data

USER pki-node
EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["pki-node"]