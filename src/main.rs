use anyhow::Result;

/// An example WebRTC server that will accept connections and run the ping protocol on them.
#[tokio::main]
async fn main() -> Result<()> {
    // spawn application as separate task
    let _handle = tokio::spawn(async {
        rtc_server::start().await.unwrap();
    });

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
