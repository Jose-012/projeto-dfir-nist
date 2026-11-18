use evtx::EvtxParser;

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let mut parser = EvtxParser::from_path(&path).expect("open");
    let mut count = 0usize;
    for record in parser.records_json_value() {
        match record {
            Ok(record) => {
                count += 1;
                if count <= 2 {
                    println!("{} {}", record.event_record_id, record.data);
                }
            }
            Err(error) => {
                let mut chain = format!("{error:?}");
                let mut source = std::error::Error::source(&error);
                while let Some(cause) = source {
                    chain.push_str(&format!("\n  caused by: {cause:?}"));
                    source = cause.source();
                }
                println!("{chain}");
                break;
            }
        }
    }
    println!("total={count}");
}
