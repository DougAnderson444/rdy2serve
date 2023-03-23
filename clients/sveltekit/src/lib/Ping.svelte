<script>
    import { onMount } from "svelte"

    // https://github.com/libp2p/js-libp2p-webrtc/tree/main/examples/browser-to-server

    // imports to connect via @libp2p/webrtc (Javascript)
    import { createLibp2p } from "libp2p"
    import { noise } from "@chainsafe/libp2p-noise"
    import { multiaddr, isMultiaddr } from "@multiformats/multiaddr"
    import { pipe } from "it-pipe"
    import { fromString, toString } from "uint8arrays"
    import { webRTC } from "@libp2p/webrtc"
    import { pushable } from "it-pushable"

    onMount(async () => {
        let stream
        let pingIntervalID

        const output = document.getElementById("output")
        const sendSection = document.getElementById("send-section")
        const appendOutput = (line) => {
            const div = document.createElement("div")
            div.appendChild(document.createTextNode(line))
            output.append(div)
        }
        const clean = (line) => line.replaceAll("\n", "")
        const sender = pushable()

        const libp2p = await createLibp2p({
            transports: [webRTC()],
            connectionEncryption: [noise()],
        })

        await libp2p.start()

        libp2p.connectionManager.addEventListener(
            "peer:connect",
            (connection) => {
                appendOutput(
                    `Peer connected '${libp2p
                        .getConnections()
                        .map((c) => c.remoteAddr.toString())}'`
                )
                sendSection.style.display = "block"
            }
        )

        window.connect.onclick = async () => {
            const ma = multiaddr(window.peer.value)
            appendOutput(`Dialing '${ma}'`)
            try {
                await libp2p.dialProtocol(ma, ["/ipfs/ping/1.0.0"])
            } catch (error) {
                console.warn("Dial failed", error)
            }

            // pipe(sender, stream, async (src) => {
            //     for await (const buf of src) {
            //         const response = toString(buf.subarray())
            //         appendOutput(`Received message '${clean(response)}'`)
            //     }
            // })

            // also ping the Server
            const doPing = async () => {
                try {
                    const latency = await libp2p.ping(ma)
                    console.log({ latency })
                } catch (error) {
                    console.warn("This ping failed", error)
                }
            }
            doPing()
            pingIntervalID = setInterval(doPing, 8000)
        }

        window.send.onclick = async () => {
            const message = `${window.message.value}\n`
            appendOutput(`Sending message '${clean(message)}'`)
            sender.push(fromString(message))
        }

        // onDestroy:
        return async () => {
            clearInterval(pingIntervalID)
            await libp2p.stop()
        }
    })
</script>

<!-- <h1>Welcome to SveleptosKit</h1>
<p>
    Below we mount the `Leptos` app, which is build with Trunk, which runs
    main() in main.rs which calls hydrate from the app library
</p> -->
<h1>Ping</h1>
<div id="app" class="m-4">
    <div class="flex items-center">
        <label for="peer">Server MultiAddress:</label>
        <input
            type="text"
            id="peer"
            class="flex-1 border p-3 mx-2 rounded font-mono text-xs"
        />
        <button
            id="connect"
            class="flex-initial border bg-green-500 rounded outline-none text-white p-2 mx-2"
            >Connect</button
        >
    </div>
    <div id="send-section" class="flex">
        <label for="message">Message:</label>
        <input
            type="text"
            id="message"
            value="hello"
            class="flex-1 border p-2 mx-2 rounded font-mono text-xs"
        />
        <button
            id="send"
            class="flex-initial border bg-green-500 rounded outline-none text-white p-2 mx-2"
            >Send</button
        >
    </div>
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
