use anyhow::Result;
use tokio::sync::oneshot;

/// An example WebRTC server that will accept connections and run the ping protocol on them.
#[tokio::main]
async fn main() -> Result<()> {
    // set up logging
    env_logger::init();
    let (sendr, recvr) = oneshot::channel::<String>();

    // spawn application as separate task
    let _handle = tokio::spawn(async move {
        rtc_server::start(sendr).await.unwrap();
    });

    // Await the response with the Address in it
    let addr = recvr.await;

    match addr {
        Ok(a) => println!("\nConnect with: \n{a}"),
        Err(e) => println!("Error getting IP multiaddress: {e}"),
    }

    // Gracefully shutdown the spawned tokio task (Ctrl+C)
    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {err}");
            // we also shut down in case of error
        }
    };

    Ok(())
}
