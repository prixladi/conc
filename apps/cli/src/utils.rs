use std::time::{SystemTime, UNIX_EPOCH};

pub fn start_time_to_age(started_time: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = match started_time >= now {
        true => 0,
        false => now - started_time,
    };

    match elapsed {
        a if a < 5 => String::from("few seconds"),
        a if a < 60 => {
            format!("{} seconds", a)
        }
        a if a < 120 => String::from("1 minute"),
        a if a < 3600 => {
            format!("{} minutes", a / 60)
        }
        a if a < 7200 => String::from("1 hour"),
        a if a < 86400 => {
            format!("{} hours", a / 3600)
        }
        a if a < 86400 * 2 => String::from("1 day"),
        a if a < 86400 * 365 => {
            format!("{} days", a / 86400)
        }
        _ => String::from("over a year"),
    }
}
