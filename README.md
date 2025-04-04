# www - MCJars Minecraft Versions Website

MCJars is a website for retrieving Minecraft server versions. It allows you to easily download, install, and lookup Minecraft server versions. This project that runs on 6 HA Hetzner VMs with 3 Load Balancers.

## Features

- Runs in Docker for high availability
- Fast Reverse Hash Lookup (< 50ms)
- Data is cached for fast repeated retrievals
- Servers in Germany, Hillsboro (Oregon, US), and Ashburn (Virginia, US)
- Blazingly 🔥 fast 🚀, written in 100% safe Rust. 🦀

## Developing

To Develop on this site, you need to install all required dependencies

```bash
git clone https://github.com/mcjars/www.git mcjars-www

cd mcjars-www/frontend

# make sure to have nodejs installed already
npm i -g pnpm
pnpm i

# build the frontend
pnpm build
cd ..

# make sure to have nodejs and rustup (cargo) installed already
cargo build

# fill out the config
cp .env.example .env

# after filling out the config
cd database
npm i -g pnpm
pnpm i

pnpm kit migrate
cd ..

# start the dev server on port 8000
cargo run
```

> [!NOTE]
> NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
