use chrono::{DateTime, Utc};

fn main() {
    let ts1 = "2025-12-02T16:28:50.113929";
    let ts2 = "2025-12-02T16:28:50.113929Z";

    println!("Without Z: {}", ts1);
    match ts1.parse::<DateTime<Utc>>() {
        Ok(dt) => println!("  OK: {}", dt),
        Err(e) => println!("  ERR: {}", e),
    }

    println!("\nWith Z: {}", ts2);
    match ts2.parse::<DateTime<Utc>>() {
        Ok(dt) => println!("  OK: {}", dt),
        Err(e) => println!("  ERR: {}", e),
    }
}
