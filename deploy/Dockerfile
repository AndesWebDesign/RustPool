FROM debian:bullseye-slim AS rustpool-build

COPY src /rustpool/src
COPY Cargo.toml /rustpool/Cargo.toml

ENV RUST_BACKTRACE=1

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        curl \
        ca-certificates \
        openssl \
        libssl-dev \
        pkg-config \
        cmake \
        build-essential \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . /root/.cargo/env \
    && cd /rustpool \
    && cargo fetch \
    && cargo build --release --package rustpool --bin rustpool


FROM debian:bullseye-slim AS rustpool

COPY --from=rustpool-build /rustpool/target/release/rustpool /usr/local/bin/rustpool
COPY deploy/files/config.json /etc/rustpool/config.json
COPY deploy/files/rustpool_entrypoint.sh /rustpool_entrypoint.sh
COPY deploy/files/wait_for_daemon.sh /wait_for_daemon.sh

ENV RUST_BACKTRACE=1
ENV RUSTPOOL_USER=rustpool_user
ENV MONERO_DAEMON_RPC_USER=MISSING
ENV MONERO_DAEMON_RPC_PASSWORD=MISSING
ENV MONERO_WALLET_RPC_USER=MISSING
ENV MONERO_WALLET_RPC_PASSWORD=MISSING

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        gosu \
    && chmod +x /usr/local/bin/rustpool \
    && chmod +x /wait_for_daemon.sh \
    && chmod +x /rustpool_entrypoint.sh \
    && adduser --disabled-login --disabled-password --shell /bin/false ${RUSTPOOL_USER} \
    && apt-get -y clean \
    && yes | apt autoremove \
    && rm -rf /var/lib/apt/lists/* \
    && unset MONERO_WALLET_RPC_USER \
    && unset MONERO_WALLET_RPC_PASSWORD \
    && unset MONERO_DAEMON_RPC_USER \
    && unset MONERO_DAEMON_RPC_PASSWORD

VOLUME /etc/rustpool

ENTRYPOINT ["rustpool_entrypoint.sh"]


FROM debian:bullseye-slim AS monero-build

ARG MONERO_URL=https://downloads.getmonero.org/cli/linux64

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
		curl \
		bzip2 \
		iproute2 \
	&& curl -kSL ${MONERO_URL} -o /monero.tar.bz2 \
	&& tar -xjf /monero.tar.bz2 --directory / \
	&& rm /monero.tar.bz2 \
	&& mv /monero-x86_64-linux-gnu-* /monero


FROM debian:bullseye-slim AS monero-daemon

ENV MONERO_USER=monero_user
ENV MONERO_DAEMON_RPC_USER=MISSING
ENV MONERO_DAEMON_RPC_PASSWORD=MISSING

COPY --from=monero-build /monero /monero
COPY deploy/files/daemon_entrypoint.sh /daemon_entrypoint.sh
COPY deploy/files/block_notify.sh /block_notify.sh

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
		gosu \
		netcat \
	&& adduser --disabled-login --disabled-password --shell /bin/false ${MONERO_USER} \
	&& install -m 0755 -o ${MONERO_USER} -g ${MONERO_USER} -t /usr/local/bin /monero/* \
	&& rm -rf /monero \
	&& chmod +x /daemon_entrypoint.sh \
	&& chmod +x /block_notify.sh \
	&& mkdir -p /var/log/monero \
	&& mkdir /monero_data \
	&& apt-get clean \
    && yes | apt autoremove \
	&& rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* \
	&& unset MONERO_DAEMON_RPC_USER \
    && unset MONERO_DAEMON_RPC_PASSWORD

VOLUME /monero_data
VOLUME /home/${MONERO_USER}/.bitmonero

EXPOSE 18081

ENTRYPOINT ["/daemon_entrypoint.sh"]


FROM debian:bullseye-slim AS monero-wallet

ENV MONERO_USER=monero_user
ENV MONERO_DAEMON_RPC_USER=MISSING
ENV MONERO_DAEMON_RPC_PASSWORD=MISSING
ENV MONERO_WALLET_RPC_USER=MISSING
ENV MONERO_WALLET_RPC_PASSWORD=MISSING
ENV POOL_WALLET_PASSWORD=MISSING

COPY --from=monero-build /monero /monero
COPY deploy/files/wallet_entrypoint.sh /wallet_entrypoint.sh
COPY deploy/files/wait_for_daemon.sh /wait_for_daemon.sh
COPY deploy/files/wait_for_database.sh /wait_for_database.sh

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
		gosu \
		curl \
		jq \
		postgresql-client \
	&& adduser --disabled-login --disabled-password --shell /bin/false ${MONERO_USER} \
	&& install -m 0755 -o ${MONERO_USER} -g ${MONERO_USER} -t /usr/local/bin /monero/* \
	&& rm -rf /monero \
	&& chmod +x /wallet_entrypoint.sh \
	&& chmod +x /wait_for_daemon.sh \
	&& chmod +x /wait_for_database.sh \
	&& mkdir /wallet_data \
	&& apt-get clean \
    && yes | apt autoremove \
	&& rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* \
	&& unset MONERO_WALLET_RPC_USER \
	&& unset MONERO_WALLET_RPC_PASSWORD \
	&& unset MONERO_DAEMON_RPC_USER \
	&& unset MONERO_DAEMON_RPC_PASSWORD \
	&& unset POOL_WALLET_PASSWORD

VOLUME /wallet_data

EXPOSE 18082

ENTRYPOINT ["/wallet_entrypoint.sh"]
