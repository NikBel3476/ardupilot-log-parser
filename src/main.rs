use flightlog_core::{Log, ReadOptions};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        panic!("Please specify filename argument");
    }
    let log_file_path = &args[1];

    let log = Log::open(log_file_path, &ReadOptions::default())
        .unwrap_or_else(|e| panic!("Cannot read {log_file_path}: {e}"));

    for table in log.tables() {
        let columns = table
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>();
        println!(
            "{:<6} {:>8} rows  {}",
            table.name,
            table.len(),
            columns.join(", ")
        );
    }
    let integrity = &log.integrity;
    println!(
        "Msg count: {}. Skipped bytes: {}",
        integrity.messages, integrity.skipped_bytes
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_fmt_and_messages_from_sample_log() {
        let log = Log::open("00000001.BIN", &ReadOptions::default()).unwrap();
        let fmt = log.table("FMT").expect("FMT messages decoded");
        assert!(fmt.len() > 0);
        let att = log
            .table("ATT")
            .expect("ATT decoded through its FMT definition");
        assert!(att.len() > 0);
        assert!(att.f64s("Roll").is_some());
        assert!(log.integrity.messages > 0);
    }
}
