#!/usr/bin/env -S just --justfile

set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

_default:
    @just --list -u

init:
    cargo install cargo-shear

ready:
    just fmt
    just check
    just fix
    just test
    just doc
    git status

check:
    cargo check --workspace --all-features --all-targets --locked

test:
    cargo test

fmt:
    cargo shear --fix
    cargo fmt --all

[unix]
doc:
    RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items

[windows]
doc:
    $Env:RUSTDOCFLAGS='-D warnings'; cargo doc --no-deps --document-private-items

fix:
    cargo fix --allow-dirty
    just fmt

run args='':
    cd playground; cargo run --release -p roan-cli run {{ args }}
