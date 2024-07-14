# www - Minecraft Jar Downloading/Lookup Website

www is a explorer-like website for downloading Minecraft server versions. It allows you to easily download, install, and lookup versions of the Minecraft server software and their Configs.

## Features

- API Ran on Cloudflare Workers
- Installation Instructions for Forge/Fabric/...
- Use Zips for Forge installations instead of installers
- Lookup Configs or Jars by dragging them in and comparing

## Developing

To Develop on this website, you need to install all required dependencies

```bash
git clone https://github.com/mcjars/www.git mcjars-www

cd mcjars-www

# make sure to have nodejs installed already
npm i -g pnpm
pnpm i

# start the dev server on port 9000
pnpm dev
```

> [!NOTE]
> NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
