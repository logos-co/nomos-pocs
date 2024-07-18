use rand::{thread_rng, Rng};
use s2n_quic::{client::Connect, Client};
use std::{
    error::Error,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
    time::{self, Duration, Instant},
};

/// NOTE: this certificate is to be used for demonstration purposes only!
pub static CERT_PEM: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/cert.pem"));

#[derive(Debug)]
struct Stat {
    duration: Duration,
}

async fn spawn_client(
    stats_tx: mpsc::Sender<Stat>,
    _id: usize,
    connection_counter: Arc<AtomicUsize>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::builder()
        .with_tls(CERT_PEM)?
        .with_io("0.0.0.0:0")?
        .start()?;

    let addr: SocketAddr = "145.239.92.79:4433".parse()?;
    let connect = Connect::new(addr).with_server_name("bacv.org");
    let mut connection = client.connect(connect).await?;

    connection.keep_alive(true)?;
    connection_counter.fetch_add(1, Ordering::SeqCst);

    let stream = connection.open_bidirectional_stream().await?;
    let (mut receive_stream, mut send_stream) = stream.split();

    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;
        let mut message = [0u8; 1024];
        thread_rng().fill(&mut message);

        let start = Instant::now();
        if let Err(e) = send_stream.write_all(&message).await {
            eprintln!("Failed to send message: {:?}", e);
            break;
        }

        let mut buf = vec![0; message.len()];
        if let Err(e) = receive_stream.read_exact(&mut buf).await {
            eprintln!("Failed to receive echo: {:?}", e);
            break;
        }

        let duration = start.elapsed();
        if stats_tx.send(Stat { duration }).await.is_err() {
            break;
        }
    }

    connection_counter.fetch_sub(1, Ordering::SeqCst);
    Ok(())
}

async fn print_stats(mut rx: mpsc::Receiver<Stat>, connection_counter: Arc<AtomicUsize>) {
    let mut total_duration = Duration::from_secs(0);
    let mut min_duration = Duration::from_secs(u64::MAX);
    let mut max_duration = Duration::from_secs(0);
    let mut count = 0;

    let mut interval = time::interval(Duration::from_secs(5));
    loop {
        tokio::select! {
            Some(stat) = rx.recv() => {
                count += 1;
                total_duration += stat.duration;
                if stat.duration < min_duration {
                    min_duration = stat.duration;
                }
                if stat.duration > max_duration {
                    max_duration = stat.duration;
                }
            }
            _ = interval.tick() => {
                let average_duration = if count == 0 {
                    Duration::from_secs(0)
                } else {
                    total_duration / count as u32
                };
                println!("Total connections: {}", connection_counter.load(Ordering::SeqCst));
                println!("Average duration: {:?}", average_duration);
                println!("Shortest duration: {:?}", min_duration);
                println!("Longest duration: {:?}", max_duration);

                total_duration = Duration::from_secs(0);
                min_duration = Duration::from_secs(u64::MAX);
                max_duration = Duration::from_secs(0);
                count = 0;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel(100);
    let connection_counter = Arc::new(AtomicUsize::new(0));

    tokio::spawn(print_stats(rx, connection_counter.clone()));

    let mut interval = time::interval(Duration::from_secs(1));
    let mut id = 0;
    loop {
        interval.tick().await;
        (0..10).for_each(|_| {
            let tx = tx.clone();
            let connection_counter = connection_counter.clone();
            id += 1;
            tokio::spawn(async move {
                let r = spawn_client(tx, id, connection_counter).await;
                if let Err(e) = r {
                    eprintln!("{e}");
                }
            });
        });
    }
}
