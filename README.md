# www - MCJars Minecraft Versions Website

MCJars is a website for retrieving Minecraft server versions. It allows you to easily download, install, and lookup Minecraft server versions. This project that runs on 6 HA Hetzner VMs with 3 Load Balancers.

## Features

- Runs in Docker for high availability
- Fast Reverse Hash Lookup (< 50ms)
- Data is cached for fast repeated retrievals
- Servers in Germany, Hillsboro (Oregon, US), and Ashburn (Virginia, US)
- Blazingly 🔥 fast 🚀, written in 100% safe Rust. 🦀

## Developing (Frontend)

The frontend is written in React + Typescript and uses Tailwind CSS for styling. It is a single page application that communicates with the backend via REST API.

### Prerequisites (Frontend)

- Node.js (v22 or higher)
- pnpm (v10 or higher)

### Getting Started (Frontend)

To get started with the frontend, you need to install all required dependencies

```bash
git clone https://github.com/mcjars/www.git mcjars-www
cd mcjars-www/frontend

# make sure to have nodejs installed already
npm i -g pnpm
pnpm i
```

### Running the Frontend

To run the frontend, you need to start the development server. This will start a local server on port 9000.

```bash
pnpm dev
```

To properly test the api without setting up the backend, you can run the following in your browser console:

```javascript
window.localStorage.setItem("api_url", "https://mcjars.app")
```

### Building the Frontend

To build the frontend for production, you need to run the following command:

```bash
pnpm build
```

This will create a production build of the frontend in the `lib` directory. You can then serve this directory with any static file server (e.g. backend).

## Developing (Backend)

The backend is written in Rust and uses Axum for the web server. It is a REST API that communicates with the database (PostgreSQL) and the frontend.

### Prerequisites (Backend)

- Rust (v1.86 or higher)
- PostgreSQL (v17)
- Node.js (v22 or higher)
- pnpm (v10 or higher)

### Getting Started (Backend)

To get started with the backend, you need to install all required dependencies. You also need to create a `.env` file in the root directory of the project.

```bash
# if you haven't already cloned
git clone https://github.com/mcjars/www.git mcjars-www
cd mcjars-www

# if you have, make sure to go into the main directory (.. if you are in frontend)

cp .env.example .env
```

You need to fill out the `.env` file with your PostgreSQL credentials and other required environment variables.

### Migrating the Database

To migrate the database, you need to setup the database and run the migrations. You can do this by running the following commands:

```bash
cd database

# make sure to have nodejs installed already
npm i -g pnpm
pnpm i

pnpm kit migrate
```

This run all migrations. Make sure to have the database running and the credentials set in the `.env` file.

### Running the Backend

To run the backend, you need to start the development server. This will start a local server on port 8000.

```bash
cargo run
```

### Building the Backend

To build the backend for production, you need to run the following command:

```bash
cargo build --release
```

> [!NOTE]
> NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
