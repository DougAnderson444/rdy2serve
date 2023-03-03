<script>
    import { onMount } from "svelte"

    // using `cargo leptos --watch` (does not work for front end)
    // import init from "../../../../target/front/wasm32-unknown-unknown/debug/leptos_start.wasm?init"

    // using `trunk serve --open --features=hydrate`
    import init from "../../../leptos/dist/leptos_start.js"

    onMount(async () => {
        init().then((instance) => {
            console.log({ instance })
            // instance.exports.hydrate()
        })
    })
</script>

<!-- <h1>Welcome to SveleptosKit</h1>
<p>
    Below we mount the `Leptos` app, which is build with Trunk, which runs
    main() in main.rs which calls hydrate from the app library
</p> -->
