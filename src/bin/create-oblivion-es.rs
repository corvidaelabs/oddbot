use clap::Parser;
use oddbot::{event_stream::create_nats_client, prelude::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the event stream
    #[arg(long)]
    stream_name: String,

    /// Comma-separated list of subjects to subscribe to
    #[arg(long, value_delimiter = ',')]
    subjects: Vec<String>,

    /// Description of the event stream
    #[arg(long)]
    description: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), OddbotError> {
    let args = Args::parse();

    let nats_client = create_nats_client().await?;

    EventStream::new_stream(
        args.stream_name,
        nats_client,
        args.subjects,
        args.description,
    )
    .await?;

    println!("Event stream created successfully");
    Ok(())
}
