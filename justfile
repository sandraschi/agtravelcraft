set windows-shell := ["powershell.exe", "-NoProfile", "-Command"]

default:
    @just --list

check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo test --workspace

fmt-check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo fmt --all --check

smoke:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p agtravelcraft-app -- --smoke

run-gui:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p agtravelcraft-app

run-mcp:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p agtravelcraft-mcp -- mcp

seed-demo:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p agtravelcraft-mcp -- demo

build-release:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo build --release --workspace
