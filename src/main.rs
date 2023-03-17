use anyhow::Result;
use rtc_server::{Message, ServerResponse};
use tokio::sync::{mpsc, oneshot};

/// An example WebRTC server that will accept connections and run the ping protocol on them.
/// Start with `carfgo run`
/// The Server will respond to channel messages with the Multiaddress of the listening protocol
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // Initiating channel, which will send a Message asking for a reply with a ServerResponse in it
    let (sendr, recvr) = mpsc::channel::<Message<ServerResponse>>(1);

    log::debug!("> Spawning task for server!");
    let _handle = tokio::spawn(async move {
        rtc_server::start(recvr).await.unwrap();
    });

    // Reply channel sends a ServerResponse back
    let (reply_sender, reply_rcvr) = oneshot::channel::<ServerResponse>();

    // Q: What happens if this request is sent before the receiver is ready?
    // A: Tokio holds all messages for us until the corresponding rcvr is setup!
    log::debug!("> Send message!");
    let _result = sendr
        .send(Message::<ServerResponse> {
            reply: reply_sender,
        })
        .await;

    if let Ok(reply) = reply_rcvr.await {
        let s: String = std::str::from_utf8(&reply.address).unwrap().into();
        // Rust doesn't support octal character escape sequence
        // For colors, use hexadecimal escape instead, plus a series of semicolon-separated parameters.
        println!("Connect with: \n\x1b[30;1;42m{s}\x1b[0m");
    }

    println!("\n*** To Shutdown, use Ctrl + C ***\n");

    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {err}");
            // we also shut down in case of error
        }
    };

    Ok(())
}
