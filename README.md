# Rdy2Serve

A [Libp2p](https://docs.rs/crate/libp2p) Webrtc Server (& chat app)

True peer to peer communications.

Browser-to-Browser coming once [the spec](https://github.com/libp2p/specs/issues/475) is complete.

## Main Package

Main package is a binary that runs the WebRTC Server.

`$ cargo run`

## RTC-Server - Web Server Library

This library is accessed by the main workspace package.

## Clients - Run Web Clients

Compile Leptos to wasm and watch using [Trunk](https://trunkrs.dev/):

```bash
$ cd clients/leptos
$ trunk watch --features=hydrate
```

Extract the Tailwindcss:

```bash
$ cd clients/leptos
$ npx tailwindcss -i ./style/input.css -o ./style/output.css  --watch
$ npx tailwindcss -i ./style/input.css -o ../sveltekit/static/output.css --watch
```

Run as a Svelte App:

```bash
$ cd clients/sveltekit
$ npm run dev
```

The client is a Leptos app which allows you to connect to the server and chat.
