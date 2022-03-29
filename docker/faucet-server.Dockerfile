FROM composablefi/rust:latest

ARG VERSION

USER root

RUN apt-get update -y && apt-get install wget curl -y --no-install-recommends \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 

RUN rustup default stable && rustup update


WORKDIR /composable

RUN wget "https://storage.googleapis.com/composable-binaries/community-releases/utils/faucet-server"  

    
RUN mv faucet-server /usr/local/bin && chmod +x /usr/local/bin/faucet-server  

EXPOSE 8088

CMD ["faucet-server", "--version"]
