<script lang="ts">
    import { onMount, onDestroy } from "svelte"

    import { createLibp2p } from "libp2p"
    import { noise } from "@chainsafe/libp2p-noise"
    import { gossipsub } from "@chainsafe/libp2p-gossipsub"
    import { multiaddr } from "@multiformats/multiaddr"

    import { webRTC } from "@libp2p/webrtc"
    import { pushable } from "it-pushable"
    import { fromString as uint8ArrayFromString } from "uint8arrays/from-string"
    import { toString as uint8ArrayToString } from "uint8arrays/to-string"
    import { peerIdFromString } from "@libp2p/peer-id"

    import type { Libp2p } from "@libp2p/interface-libp2p"
    import type { Multiaddr, MultiaddrInput } from "@multiformats/multiaddr"
    import type { PeerId } from "@libp2p/interface-peer-id"

    const topic = "chat"
    let message = "Hello"
    let pingIntervalID: NodeJS.Timer
    let libp2p: Libp2p
    let ma: Multiaddr
    let libp2pReady: boolean
    let handleConnect: () => Promise<void>
    let handleSend: () => Promise<void>
    let peer: MultiaddrInput | undefined
    let remotePeerId: PeerId
    let connectable: boolean = true
    let handshook: boolean = false

    const destroy = async () => {
        clearInterval(pingIntervalID)
        if (libp2p) await libp2p.stop()
    }

    onDestroy(destroy)

    onMount(async () => {
        let period = 30000
        let retries = 3
        let retryPeriod = 5000

        const output = document.getElementById("output")
        const sendSection = document.getElementById("send-section")
        const appendOutput = (line: string) => {
            const div = document.createElement("div")
            div.appendChild(document.createTextNode(line))
            output?.append(div)
        }
        const clean = (line: string) => line.replaceAll("\n", "")
        const sender = pushable()

        // networking debug logs
        localStorage.setItem(
            "debug",
            "libp2p:webrtc:connection,libp2p:webrtc:transport,libp2p:webrtc:stream,libp2p:dialer,libp2p:webrtc:sdp"
        )

        libp2p = await createLibp2p({
            transports: [webRTC()],
            connectionEncryption: [noise()],
            pubsub: gossipsub({
                allowPublishToZeroPeers: true,
                // emitSelf: true,
                // directPeers: [{ id: remotePeerId, addrs: [ma] }],
            }),
        })

        await libp2p.start()
        await libp2p.pubsub.subscribe(topic)

        const doPing = async () => {
            try {
                const latency = await libp2p.ping(ma)
                console.log({ latency })
            } catch (error) {
                console.warn("This ping failed", error)
                clearInterval(pingIntervalID)
            }
            let msg = "Bird bird bird, bird is the word!"
            console.log(`Sending msg: ${msg}`)
            libp2p.pubsub
                .publish(topic, uint8ArrayFromString(msg))
                .catch((err) => {
                    console.error(err)
                })
        }

        function retry() {
            let conns = libp2p.getConnections()
            console.log({ conns })
            // close the connection to the failed peer
            closeConnections(remotePeerId)
            // trigger another dial attempt
            tryDial()
        }

        function closeConnections(peerId: PeerId) {
            // @ts-ignore
            libp2p.connectionManager.closeConnections(peerId)
        }
        /**
         * If this web client doesn't rx a message shortly after connecting
         * retry the connection
         * (close the connection and redial)
         */
        libp2p.addEventListener("peer:connect", async (connection) => {
            appendOutput(
                `Peer connected '${libp2p
                    .getConnections()
                    .map((c) => c.remoteAddr.toString())}'`
            )
            // wait a sec to see if we got a handshale msg from the Server
            // await wait(retryPeriod)
            // if (!handshook) {
            //     retry()
            //     return
            // }

            if (sendSection) sendSection.style.display = "block" // show the message section
            // doPing()
        })

        libp2p.addEventListener("peer:disconnect", (connection) => {
            console.log("Disconected", { connection })
            clearInterval(pingIntervalID)

            appendOutput(`Peer disconnected`)
            if (sendSection) sendSection.style.display = "none"
        })

        libp2p.pubsub.addEventListener("message", (evt) => {
            handshook = true
            console.log({ evt })
            let msg = `*** Gossipsub received: ${uint8ArrayToString(
                evt.detail.data
            )} on topic ${evt.detail.topic}, sending replies`
            console.log(msg)
            appendOutput(msg)
            doPing()
            pingIntervalID = setInterval(doPing, period)
        })

        const tryDial = async () => {
            /**
             * protocols: [
             * "/floodsub/1.0.0", // do not use
             * "/ipfs/id/1.0.0",
             * "/ipfs/id/push/1.0.0",
             * "/ipfs/ping/1.0.0",
             * "/libp2p/circuit/relay/0.1.0",
             * "/libp2p/fetch/0.0.1",
             * "/meshsub/1.0.0", // gossipsub
             * "/meshsub/1.1.0"] // included by defalt in rust-libp2p
             * */
            connectable = false
            try {
                await libp2p.dialProtocol(ma, [
                    "/meshsub/1.1.0",
                    "/ipfs/ping/1.0.0",
                ])
            } catch (error) {
                console.warn("Dial did not go as planed. Retry?", error)
                // retry()
            }
        }

        handleConnect = async () => {
            ma = multiaddr(peer)
            appendOutput(`Dialing...`)

            if (ma.getPeerId()) {
                // @ts-ignore
                remotePeerId = peerIdFromString(ma.getPeerId())
                await libp2p.peerStore.addressBook.set(remotePeerId, [ma])
            }
            tryDial()
        }

        handleSend = async () => {
            appendOutput(`Sending message '${clean(message)}'`)
            libp2p.pubsub
                .publish(topic, uint8ArrayFromString(clean(message)))
                .catch((err) => {
                    console.error(err)
                })
        }

        libp2pReady = true
        return destroy
    })

    function wait(ms: number) {
        return new Promise((resolve) => setTimeout(resolve, ms))
    }
</script>

<!-- <h1>Welcome to SveleptosKit</h1>
<p>
    Below we mount the `Leptos` app, which is build with Trunk, which runs
    main() in main.rs which calls hydrate from the app library
</p> -->
<h1 class="text-3xl m-2 font-bold">Chat</h1>
<span class="text-lg m-2 font-bold">{libp2p?.peerId}</span>

<div id="app" class="m-4">
    <form class="flex items-center" on:submit|preventDefault={handleConnect}>
        <label for="peer">Server MultiAddress:</label>
        <input
            type="text"
            id="peer"
            bind:value={peer}
            class="flex-1 border p-3 mx-2 rounded font-mono text-xs"
        />
        {#if libp2pReady}
            <input
                disabled={!connectable}
                type="submit"
                id="connect"
                value="Connect"
                class="flex-initial border bg-green-500 rounded outline-none text-white p-2 mx-2"
            />
        {/if}
    </form>
    <form id="send-section" class="flex" on:submit|preventDefault={handleSend}>
        <label for="message">Message:</label>
        <input
            type="text"
            bind:value={message}
            id="message"
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
    label {
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
