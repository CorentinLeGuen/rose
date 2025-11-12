# :rose: Rose

Rose (Rust Object storage Env) is a service based on [_object_store_](https://docs.rs/object_store/latest/object_store/) to keep track of space used.

[![Project Status](https://img.shields.io/badge/status-in%20development-orange?style=for-the-badge)](https://github.com/CorentinLeGuen/nuage/tags)

## :package: Using Cargo

Database migrations: `cargo run --bin migrate` (currently using cockroachdb).

You must copy the [.env.example](.env.example) into a .env file and update credentials.

`cargo build --release` and then [rose app should be available](./target/release/rose).

## Current features

- Basic GET, PUT, HEAD and DELETE endpoints
- Database schema migrations

## TODO

- :shipit: more features ...
- HEAD: get real metadata instead of just a HTTP status code
- managing versionned buckets
- :whale: Set up a container
- add a database
- multipart uploads
- tests