# :rose: Rose

Rose (Rust Object Storage Endpoints) is an extra layer API of Object storage services that implements [AWS S3 SDK protocol](https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/) and use [cockroachDB](https://www.cockroachlabs.com/) to keep track of space used.

[![Project Status](https://img.shields.io/badge/status-in%20development-orange?style=for-the-badge)](https://github.com/CorentinLeGuen/nuage/tags)

## Requirements

### Database

Rose is using [cockroachDB](https://www.cockroachlabs.com/) as database.
You can setup a simple cockroach container with `docker run -d --name rosedb -p 26257:26257 cockroachdb/cockroach:latest start-single-node --insecure` then run migrations with `cargo run --bin migrate` to create tables.

### Object Storage

Rose needs a Bucket (Object Storage). I am using a basic [lifecycle policy](lifecycle-policy.json) to cleanup deleted files after 30 days and aborted multipart uploads after 7 days.

### Build & Run

Set your access keys and credentials: `cp .env.example .env`.

And build the app with `cargo build --release`, and then [rose app should be available here](./target/release/rose).

## Features

### Current features

- Basic GET, PUT, HEAD and DELETE endpoints
- *aws_sdk_s3* compatible storage
- Database schema migrations

### TODO

- :shipit: more features ...
- http requests collection to test endpoints ? (.http scripts)
- add "how to use" section in Readme
- update [lifecycle policy](lifecycle-policy.json) with something more robust but open to any S3 compatible bucket
- migration: set database url from config
- FAQ ?
- managing versionned buckets
- :whale: Set up Rust as a container
- multipart uploads
- tests

## Contact

Any question or suggestion, you can contact me at this address [leguen.corentin@pm.me](mailto:leguen.corentin@pm.me)
