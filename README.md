# Icy

## Compiling for Raspberry Pi

Use [cross](https://github.com/rust-embedded/cross) for cross compiling to raspberry pi.
Using just cargo --target=armv7-unknown-linux-gnueabihf would be possible if there were none C dependencies, but Diesel depends on libpq.

1. Install cross: `cargo install cross`
1. Download cross source: `git clone https://github.com/rust-embedded/cross.git`
1. Modify armv7-unknown-linux-gnueabihf Dockerfile, [see](cross-changes.patch).
   - Change Ubuntu to Debian Stretch
   - Enable Debian multiarch for armhf arch.
   - Install armhf version of libpq-dev.
1. Build the modifed Dockerfile `./build-docker-image.sh armv7-unknown-linux-gnueabihf`
1. Compile with cross: `cross build --target armv7-unknown-linux-gnueabihf --release`

## Creating a deb package

[cargo-deb](https://github.com/mmstick/cargo-deb) is used for creating a .deb file.

1. Install cargo-deb: `cargo install cargo-deb`
1. Create a .deb file from binary built with cross `cargo deb --no-build --target armv7-unknown-linux-gnueabihf`
1. Copy target to pi. `scp target/armv7-unknown-linux-gnueabihf/debian/<name>.deb <address>:.`
1. Install `dpkg -i <name>.deb`
1. Configure systemd service file on pi.

## Setup Postgresql

```sh
sudo -u postgres psql
```

Create application user and database.

```sql
create database measurements;
create user measurements;
alter user measurements with password 'pass';
grant all privileges on database measurements to measurements;
```

## Run migrations 

Install diesel cli: https://github.com/diesel-rs/diesel/tree/master/diesel_cli

Locally:

```sh
diesel migration run --database-url postgresql://measurements:pass@localhost/measurements
```

On remote server:

```sh
ssh -L 9000:localhost:5432 example.com
# Open a new shell
diesel migration run --database-url postgresql://measurements:pass@localhost:9000/measurements
```
