<script>
    // @ts-nocheck

    import { onMount, onDestroy } from "svelte"

    // imports to connect via @libp2p/webrtc (Javascript)
    import { createLibp2p } from "libp2p"
    import { noise } from "@chainsafe/libp2p-noise"
    import { gossipsub } from "@chainsafe/libp2p-gossipsub"
    import { floodsub } from "@libp2p/floodsub"
    import { multiaddr, isMultiaddr } from "@multiformats/multiaddr"
    import { pipe } from "it-pipe"
    import { fromString, toString } from "uint8arrays"
    import { webRTC } from "@libp2p/webrtc"
    import { pushable } from "it-pushable"
    import { fromString as uint8ArrayFromString } from "uint8arrays/from-string"
    import { toString as uint8ArrayToString } from "uint8arrays/to-string"
    import { peerIdFromString } from "@libp2p/peer-id"

    const topic = "chat"
    let pingIntervalID
    let libp2p
    let ma
    let stream
    let libp2pReady
    let handleConnect
    let onSubmit

    const destroy = async () => {
        clearInterval(pingIntervalID)
        if (stream) stream.close()
        if (libp2p) await libp2p.stop()
    }

    onDestroy(destroy)

    onMount(async () => {
        console.log("Mounted")
        let period = 5000

        const output = document.getElementById("output")
        const sendSection = document.getElementById("send-section")
        const appendOutput = (line) => {
            const div = document.createElement("div")
            div.appendChild(document.createTextNode(line))
            output.append(div)
        }
        const clean = (line) => line.replaceAll("\n", "")
        const sender = pushable()

        libp2p = await createLibp2p({
            transports: [webRTC()],
            connectionEncryption: [noise()],
            // we add the Pubsub module we want
            pubsub: gossipsub({
                allowPublishToZeroPeers: true,
                // emitSelf: true,
                // directPeers: [{ id: remotePeerId, addrs: [ma] }],
            }),
        })

        await libp2p.start()
        await libp2p.pubsub.subscribe(topic)

        const doPing = async () => {
            // sender.push(fromString("a message from JS"))
            // try {
            //     const latency = await libp2p.ping(ma)
            //     console.log({ latency })
            // } catch (error) {
            //     console.warn("This ping failed", error)
            //     clearInterval(pingIntervalID)
            //     // pingIntervalID = setInterval(doPing, period)
            // }
            let msg = "Bird bird bird, bird is the word!"
            console.log(`Sending msg: ${msg}`)
            libp2p.pubsub
                .publish(topic, uint8ArrayFromString(msg))
                .catch((err) => {
                    console.error(err)
                })
        }

        libp2p.addEventListener("peer:connect", async (connection) => {
            console.log("peer connected", { connection })
            appendOutput(
                `Peer connected '${libp2p
                    .getConnections()
                    .map((c) => c.remoteAddr.toString())}'`
            )
            sendSection.style.display = "block"
            await libp2p.pubsub.subscribe(topic)
            doPing()
        })

        libp2p.addEventListener("peer:disconnect", (connection) => {
            console.log("Disconected", { connection })
            clearInterval(pingIntervalID)

            appendOutput(`Peer disconnected`)
            sendSection.style.display = "none"
        })

        libp2p.pubsub.addEventListener("message", (evt) => {
            console.log({ evt })
            let msg = `*** Gossipsub received: ${uint8ArrayToString(
                evt.detail.data
            )} on topic ${evt.detail.topic}, sending replies`
            console.log(msg)
            appendOutput(msg)
            // pingIntervalID = setInterval(doPing, period)
        })

        handleConnect = async () => {
            ma = multiaddr(window.peer.value)
            // appendOutput(`Dialing '${ma}'`)

            const remotePeerId = peerIdFromString(ma.getPeerId())

            await libp2p.peerStore.addressBook.set(remotePeerId, [ma])

            /**
             * protocols: [
             * "/floodsub/1.0.0", // do not use
             * "/ipfs/id/1.0.0",
             * "/ipfs/id/push/1.0.0",
             * "/ipfs/ping/1.0.0",
             * "/libp2p/circuit/relay/0.1.0",
             * "/libp2p/fetch/0.0.1",
             * "/meshsub/1.0.0",
             * "/meshsub/1.1.0"]
             * */
            try {
                stream = await libp2p.dialProtocol(ma, [
                    // "/ipfs/ping/1.0.0",
                    "/meshsub/1.1.0",
                ])
                console.log({ stream }) // WebRTC Stream
            } catch (error) {
                console.warn("Dial failed", error)
            }

            // await libp2p.pubsub.subscribe(topic)

            // stream = await libp2p.dialProtocol(ma, [
            //     "/ipfs/ping/1.0.0",
            //     // "/ipfs/id/1.0.0",
            //     // "ipfs/0.1.0",
            //     // "/gossipsub/1.0.0",
            // ])

            // pipe(sender, stream, async (src) => {
            //     for await (const buf of src) {
            //         const response = toString(buf.subarray())
            //         appendOutput(`Received message '${clean(response)}'`)
            //     }
            // })
        }

        onSubmit = async () => {
            const message = `${window.message.value}\n`
            appendOutput(`Sending message '${clean(message)}'`)
            libp2p.pubsub
                .publish(topic, uint8ArrayFromString(clean(message)))
                .catch((err) => {
                    console.error(err)
                })
        }
        console.log("Ready")

        libp2pReady = true
        return destroy
    })
</script>

<!-- <h1>Welcome to SveleptosKit</h1>
<p>
    Below we mount the `Leptos` app, which is build with Trunk, which runs
    main() in main.rs which calls hydrate from the app library
</p> -->
<h1 class="text-3xl m-2 font-bold">Chat {libp2p?.peerId}</h1>

<div id="app" class="m-4">
    <div class="flex items-center">
        <label for="peer">Server MultiAddress:</label>
        <input
            type="text"
            id="peer"
            class="flex-1 border p-3 mx-2 rounded font-mono text-xs"
        />
        {#if libp2pReady}
            <button
                id="connect"
                on:click={handleConnect}
                class="flex-initial border bg-green-500 rounded outline-none text-white p-2 mx-2"
                >Connect</button
            >
        {/if}
    </div>
    <form id="send-section" class="flex" on:submit|preventDefault={onSubmit}>
        <label for="message">Message:</label>
        <input
            type="text"
            id="message"
            value="hello"
            class="flex-1 border p-2 mx-2 rounded font-mono text-xs"
        />
        <input
            type="submit"
            id="send"
            value="Send"
            class="flex-initial border bg-green-500 rounded outline-none text-white p-2 mx-2"
        />
    </form>
    <div id="output" />
</div>

<style>
    label,
    button {
        display: block;
        font-weight: bold;
        margin: 5px 0;
    }
    div {
        margin-bottom: 20px;
    }
    #send-section {
        display: none;
    }
    input[type="text"] {
        width: 800px;
    }
</style>
