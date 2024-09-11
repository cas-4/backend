# CAS4 backend API

<img src="https://avatars.githubusercontent.com/u/175958109?s=100&v=4" alt="Logo" align="right"/>

This repo refers to a GraphQL API written in Rust used for a project for the
[Context Aware System](https://www.unibo.it/en/study/phd-professional-masters-specialisation-schools-and-other-programmes/course-unit-catalogue/course-unit/2023/479036)
class at the [University of Bologna](https://unibo.it).

## Development

You need:
- Rust [`>=1.81.0`](https://github.com/rust-lang/rust/releases/tag/1.81.0).
- `libssl-dev` in Debian-based environment. `sudo apt install libssl-dev`.
- [PostgreSQL](https://www.postgresql.org/).
- [PostGIS](https://postgis.net/) extension.

Now you set up some env variables:

- `RUST_LOG`: used by the logger.

- `DATABASE_URL`: it can be in a DSN format such as `host=localhost
  user=postgres password=password dbname=cas4 port=5432` or in a URL format such
  as `postgres://postgres:password@localhost:5432/cas4`.

- `JWT_SECRET`: this _must_ be secret because it is used to encrypt/decrypt JWT
  tokens.

- `ALLOWED_HOST`: refers to the online host of the service (eg: `0.0.0.0:8000`).

- `EXPO_ACCESS_TOKEN`: used by the [Expo](https://expo.dev) API access.

After that you must copy the `schema/init.sql` file into the database.

Now just run the app

```text
cargo run
```

## Deploy

Fortunately the deployment is automatized by the GitHub Action `cd.yml` which
pushes the latest release version to a [GHCR.io package](https://github.com/cas-4/backend/pkgs/container/backend).

A new version is released using

```text
./scripts/release X.Y.Z
```

Now you just exec

```text
docker pull ghcr.io/cas-4/backend:latest
```

Or you can build a new image

```text
docker build -t cas:latest .
docker run \
    -e RUST_LOG=... \
    -e DATABASE_URL=... \
    -e JWT_SECRET=... \
    -e ALLOWED_HOST=... \
    -e EXPO_ACCESS_TOKEN ... \
    cas:latest
```

Or the Docker compose which puts up also the PostgreSQL locally.

```text
docker compose up
```

### Kubernetes

If you do not want to use Docker or Docker compose directly, you can use a
Kubernetes cluster like [MiniKube](https://minikube.sigs.k8s.io/docs/).

```text
./scripts/k8s (apply|delete)
```

## Documentation

An always updated documentation is available at [this link](https://cas-4.github.io/backend/cas/index.html).
