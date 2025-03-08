#!/usr/bin/env just --justfile

release:
  cargo build --release    

lint:
  cargo clippy --all-targets --workspace --all-features

lint-fix:
  cargo clippy --fix --all-targets --allow-dirty --allow-staged --workspace --all-features

fmt:
  cargo fmt --all

check:
  cargo check --workspace

[private]
pre-build: check

example name:
  cargo run --example {{name}}

[confirm("Run a dry run of publishing all crates in the workspace to the `gitea` registry? Y/n")]
publish-gitea-dry: pre-build lint
  cd sqlx-utils-macro
  cargo publish --registry gitea --dry-run
  cd ..
  cargo publish --registry gitea --dry-run

[confirm("Run a dry run of publishing all crates in the workspace? Y/n")]
publish-dry: pre-build lint
  cd sqlx-utils-macro
  cargo publish --dry-run
  cd ..
  cargo publish --dry-run

[confirm("Are you sure you want to publishing all crates in the workspace to the `gitea` registry? Y/n")]
publish-gitea: publish-gitea-dry
  cd sqlx-utils-macro
  cargo publish --registry gitea
  cd ..
  cargo publish --registry gitea

[confirm("Are you sure you want to publishing all crates in the workspace? Y/n")]
publish: publish-dry
  cd sqlx-utils-macro
  cargo publish
  cd ..
  cargo publish
