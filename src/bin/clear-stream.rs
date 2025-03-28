use async_nats::jetstream::consumer::DeliverPolicy;
use clap::Parser;
use futures::StreamExt;
use oddbot::{nats::create_nats_client, prelude::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the event stream to clear
    #[arg(long)]
    stream_name: String,

    /// Force deletion without confirmation
    #[arg(long)]
    force: bool,

    /// Batch size for message deletion
    #[arg(long, default_value = "100")]
    batch_size: usize,
}

#[tokio::main]
async fn main() -> Result<(), OddbotError> {
    let args = Args::parse();

    if !args.force {
        println!(
            "Warning: This will delete all messages in the stream '{}'",
            args.stream_name
        );
        println!("Are you sure you want to continue? [y/N]");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Could not read input");

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    let nats_client = create_nats_client().await?;
    let event_stream = EventStream::connect(args.stream_name, nats_client).await?;

    // Create a temporary consumer that will read all messages
    let consumer = event_stream
        .create_consumer(None, "oddlaws.events.>".into(), Some(DeliverPolicy::All))
        .await?;

    let mut total_messages = 0;

    loop {
        let mut messages = consumer
            .fetch()
            .max_messages(args.batch_size)
            .messages()
            .await?;

        let mut batch_count = 0;
        while let Some(message) = messages.next().await {
            let Ok(message) = message else {
                continue;
            };

            // Delete the message by acknowledging it
            message.ack().await.expect("Could not acknowledge message");
            batch_count += 1;
            total_messages += 1;
        }

        println!("Cleared {} messages", total_messages);

        if batch_count < args.batch_size {
            break; // No more messages
        }

        // Add a small delay between batches
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    println!(
        "Successfully cleared {} messages from the stream",
        total_messages
    );
    Ok(())
}
