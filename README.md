# Rdy2Serve

## Rust-Libp2p Server + JS-libp2p browser client over WebRTC + Gossipsub.

Work in progress. Contributions welcome.

A [Libp2p](https://docs.rs/crate/libp2p) WebRTC Server (& budding chat app), destined to become Browser-to-Browser coming once [the spec](https://github.com/libp2p/specs/issues/475) is complete.

## Main Application (Binary)

1. Start the server. Main package is a binary that runs the WebRTC Server:

`$ cargo run`

## Svelte Web Client

2. Start the web client:

```cli
$ cd clients/sveltekit
$ npm run dev
```
