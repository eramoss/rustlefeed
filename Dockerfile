FROM debian:buster-slim as runner
RUN apt-get update && apt-get install -y curl ca-certificates libssl-dev gcc pkg-config libsqlite3-dev 

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

COPY . .
RUN cargo build --release
CMD [ "cargo" , "run" , "--release"]