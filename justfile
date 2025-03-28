set dotenv-load := true

dev:
    watchexec -i ".github/**" -i "k8s/**" -i "target/**" just start-bot

build:
    cargo build

build-image:
    docker build -t oddlaws-bot .

create-stream:
    cargo run --bin create-oblivion-es -- --stream-name="ODDLAWS_EVENTS" --subjects="oddlaws.events.>" --description="Oddlaws Event Stream"

start-server:
    cargo run --bin oblivion-server

start-bot:
    cargo run --bin oddbot


start-oblivion-server:
    cargo run --bin oblivion-server

dev-server:
    watchexec -i ".github/**" -i "k8s/**" -i "target/**" just start-oblivion-server

clear-stream:
    nats stream purge ODDLAWS_EVENTS
