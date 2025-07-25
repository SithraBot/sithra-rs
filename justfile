default:
    just --list

example_server:
    cargo build -p sithra-server --example server_a
    cargo build -p sithra-server --example server_b
    cd target/debug/examples && ./server_a

typeshare:
    typeshare . --lang=typescript -d=sithra-js/types/src

run:
    cargo build --all
    cargo run

run_web:
    (cd crates/sithra-web/web && npm run build)
    cargo build --all
    cp -rf crates/sithra-web/web/build web
    cargo run -psithra-web

build_linux_x86_64:
    cargo build --all -r --target x86_64-unknown-linux-musl

init:
    (cd crates/sithra-web/web && npm install)
