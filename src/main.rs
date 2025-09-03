use std::time::{SystemTime, UNIX_EPOCH};

use tokio::spawn;
use tokio::time::{Duration, sleep};
use tokio_serial::SerialStream;
use log::{info, error, debug};

use bridge::ioevent::IoEvent;
use host::output_feeder::OutputFeeder;
use host::serial_host;
use serial_host::SerialHost;
use iojss::journal::{EntryJournal, Telemetry};

// 120 MHz clock period in nanoseconds (1/120_000_000 seconds = 8.333... ns)
const SAMPLE_CLOCK_PERIOD_NS: f64 = 1_000_000_000.0 / 120_000_000.0;


#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let ser_port: SerialStream = serial_host::init().await;

    // Initialize the EntryJournal
    let (_conn, entry_journal) = EntryJournal::initialize("telemetry.db").await?;

    let (mut host, event_sender, mut event_receiver) = SerialHost::new(ser_port);
    let _usb_task = spawn(async move { host.serial_protocol().await });

    let mut feeder = OutputFeeder::new(event_sender);
    let mut _csv_events: Vec<IoEvent> = feeder.feed_from_csv().await;

    info!("Begin test");

    // Capture the start time of event processing
    let capture_start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let _receive_task = spawn(async move {
        loop {
            let mut input_events: Vec<IoEvent> = Vec::new();
            event_receiver.recv_many(&mut input_events, 2048).await;
            debug!("Received {} events", input_events.len());
            for event in input_events.drain(..) {
                // Convert IoEvent to Telemetry and store in EntryJournal
                match event {
                    IoEvent::DigitalEvent(_data, sample_count) => {
                        // Calculate timestamp based on sample_count and clock period
                        let elapsed_ns = (sample_count as f64 * SAMPLE_CLOCK_PERIOD_NS) as u64;
                        let event_timestamp_ns = capture_start_time + elapsed_ns;
                        // Convert to seconds for the telemetry entry
                        let timestamp = event_timestamp_ns / 1_000_000_000;
                        
                        // Extract values from IoEvent
                        let state = event.read_data().unwrap_or(0) as u64;
                        let source = event.read_slot().unwrap_or(0) as u64;
                        let register = sample_count as u64;
                        
                        // Create and insert telemetry entry
                        let telemetry = Telemetry::new(timestamp, state, source, register);
                        match entry_journal.insert_telemetry(&telemetry).await {
                            Ok(_) => debug!("Stored telemetry: time={}, state={}, source={}, register={} (sample_count={})", 
                                            timestamp, state, source, register, sample_count),
                            Err(e) => error!("Telemetry insert error: {e}"),
                        }
                    },
                    _ => {
                        debug!("Skipped non-digital event: {:?}", event);
                    }
                }
            }
            debug!("Recorded received events as telemetry");
        }
    });
    sleep(Duration::from_secs(10)).await;
    
    // Verify stored telemetry entries by creating a new journal connection
    info!("Verifying stored telemetry");
    let (_verify_conn, verify_journal) = EntryJournal::initialize("telemetry.db").await?;
    let telemetry_entries = verify_journal.get_telemetry().await?;
    info!("Found {} telemetry entries in database", telemetry_entries.len());
    
    for (idx, entry) in telemetry_entries.iter().enumerate() {
        if idx < 5 {
            info!("Entry {}: time={}, state={}, source={}, register={}", 
                    idx + 1, entry.time, entry.state, entry.source, entry.register);
        } else if idx == 5 {
            info!("... and {} more entries", telemetry_entries.len() - 5);
            break;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use iojss::journal::{EntryJournal, Telemetry};

    #[tokio::test]
    async fn test_telemetry_single_insert() -> Result<(), Box<dyn std::error::Error>> {
        let (_conn, entry_journal) = EntryJournal::initialize(":memory:").await?;

        let telemetry = Telemetry::new(1625097600, 42, 1, 255);

        entry_journal.insert_telemetry(&telemetry).await?;

        let telemetry_entries = entry_journal.get_telemetry().await?;
        assert_eq!(telemetry_entries.len(), 1);
        assert_eq!(telemetry_entries[0].time, 1625097600);
        assert_eq!(telemetry_entries[0].state, 42);
        assert_eq!(telemetry_entries[0].source, 1);
        assert_eq!(telemetry_entries[0].register, 255);

        Ok(())
    }
}
