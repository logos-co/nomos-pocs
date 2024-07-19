use s2n_quic::Server;
use std::error::Error;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// NOTE: this certificate is to be used for demonstration purposes only!
pub static CERT_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cert.pem"));
/// NOTE: this certificate is to be used for demonstration purposes only!
pub static KEY_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/key.pem"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection_counter = Arc::new(AtomicUsize::new(0));
    let byte_counter = Arc::new(AtomicUsize::new(0));

    let connection_counter_a = connection_counter.clone();
    let byte_counter_b = byte_counter.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            println!(
                "Total connections: {}",
                connection_counter_a.load(Ordering::SeqCst)
            );
            println!(
                "Total bytes received: {}",
                byte_counter_b.load(Ordering::SeqCst)
            );
        }
    });

    let mut server = Server::builder()
        .with_tls((CERT_PEM, KEY_PEM))?
        .with_io("0.0.0.0:4433")?
        .start()?;

    while let Some(mut connection) = server.accept().await {
        let connection_counter = connection_counter.clone();
        let byte_counter = byte_counter.clone();
        connection_counter.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(async move {
            while let Ok(Some(mut stream)) = connection.accept_bidirectional_stream().await {
                let byte_counter = byte_counter.clone();
                tokio::spawn(async move {
                    while let Ok(Some(data)) = stream.receive().await {
                        byte_counter.fetch_add(data.len(), Ordering::SeqCst);
                        stream.send(data).await.expect("stream should be open");
                    }
                });
            }
        });
    }

    Ok(())
}
