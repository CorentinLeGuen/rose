# :rose: Rose

Rose (Rust Object Storage Env) is a service based on [_object_store_](https://docs.rs/object_store/latest/object_store/) to keep track of space used.

[![Project Status](https://img.shields.io/badge/status-in%20development-orange?style=for-the-badge)](https://github.com/CorentinLeGuen/nuage/tags)

## Requirements

### Database

Rose is using [cockroachDB](https://www.cockroachlabs.com/) as database and migrations are made with `cargo run --bin migrate`.

You can setup a cockroach container with `docker run -d --name rosedb -p 26257:26257 cockroachdb/cockroach:latest start-single-node --insecure` and, then, set up database with `cargo run --bin migrate` to apply database migrations.

### Object Storage

Rose needs a Bucket (Object Storage). I am using a basic [lifecycle policy](lifecycle-policy.json) to cleanup deleted files after 30 days and aborted multipart uploads after 7 days.

### Build & Run

Set your access keys and credentials: `cp .env.example .env`.

And build the app with `cargo build --release`, and then [rose app should be available here](./target/release/rose).

## Current features

- Basic GET, PUT, HEAD and DELETE endpoints
- Database schema migrations

## TODO

- :shipit: more features ...
- http requests collection ? JSON or .http scripts
- managing versionned buckets
- :whale: Set up Rust as a container
- multipart uploads
- tests

## Contact

Any question or suggestion, you can contact me at this address [leguen.corentin@pm.me](mailto:leguen.corentin@pm.me)