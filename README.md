# Icy

System for reading measurements with [Bosch BME280](https://www.bosch-sensortec.com/bst/products/all_products/bme280) sensors, sending them to central MQTT server and displaying recorded measurements on a webpage.

It is meant to be installed on raspberry pi running raspbian.

This repository contains client program for pi and esp8622 microcontroller and server program for pi. Clients collect measurements, server stores them on a database and serves them on a webpage.

## Compiling for Raspberry Pi

How to compile icy-server and icy-client to pi.

Use [cross](https://github.com/rust-embedded/cross) for cross compiling to raspberry pi.
Using just cargo --target=armv7-unknown-linux-gnueabihf would be possible if there were none C dependencies, but Diesel depends on libpq.

1. Install cross: `cargo install cross`
1. Download cross source: `git clone https://github.com/rust-embedded/cross.git`
1. Modify armv7-unknown-linux-gnueabihf Dockerfile, [see](cross-changes.patch) for changes.
   - Run `git apply cross-changes.patch` in cross repo to copy changes.
   Changes:
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
1. Modify configure file on pi.
   `/usr/local/etc/icy-client/icy-client.toml` or `/usr/local/etc/icy-server/icy-server.toml` 

## Setup Postgresql

```sh
apt install postgresql
sudo -u postgres psql
```

Create application user and database.

```sql
create database measurements;
create user measurements;
alter user measurements with password 'pass';
grant all privileges on database measurements to measurements;
```

### Run migrations 

Install diesel cli: https://github.com/diesel-rs/diesel/tree/master/diesel_cli

Change to icy-server directory.

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
## Setup Mosquitto

```sh
apt install mosquitto
```

Configure user for server and clients with `mosquitto_passwd`.
