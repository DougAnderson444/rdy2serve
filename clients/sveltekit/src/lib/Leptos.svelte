<script>
    import { onMount } from "svelte"

    // When using `cargo leptos --watch` (does not work for front end)
    // import init from "../../../../target/front/wasm32-unknown-unknown/debug/leptos_start.wasm?init"

    // When using `trunk serve --open --features=hydrate` (works)
    import init from "../../../leptos/dist/leptos_start.js"

    onMount(async () => {
        await init()
    })

    </script>