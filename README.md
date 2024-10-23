# Crash Server

Proof of concept of crash casino game server using  "provably fair" system.

Game client: https://github.com/goldenratio/crash-client-react

### Local Dev Guide

- `nix develop`
- copy/paste `.env.example` to `.env`
- `cargo run`

### Create Build

- Build docker image, `./build-docker-image.sh`
- Run image, `./run-docker-image.sh`


### Create Build via Nix Flake

#### For Rust Binaries

`nix build`

- creates binary in `result/bin`
- run it with env variable, `RUST_LOG=debug ./result/bin/crash-server` for logs

#### For Docker Image

- `nix build .#dockerImage`
- `docker load < result`
- Run image,`./run-docker-image.sh`
