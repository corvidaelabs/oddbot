set dotenv-load := true

dev:
    watchexec -i ".github/**" -i "k8s/**" -i "target/**" cargo run

build:
    cargo build

build-image:
    docker build -t oddlaws-bot .
