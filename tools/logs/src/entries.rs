use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use tracing::Level;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: Level,
    pub target: String,
    pub message: String,
    pub module: String,
    pub file: String,
}

impl LogEntry {
    pub fn from_string(s: &str) -> Self {
        let log_regex = Regex::new(
            r"(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}) (?P<level>[A-Z]+) (?P<module>[^\s]+): (?P<file>[^:]+):(?P<line>\d+): (?P<message>.+)"
        ).unwrap();

        if let Some(caps) = log_regex.captures(s) {
            let timestamp_str = &caps["timestamp"];
            let naive_dt = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.3f")
                .expect("Failed to parse timestamp");
            let timestamp = DateTime::<Utc>::from_utc(naive_dt, Utc);

            let level_str = &caps["level"];
            let level = match level_str {
                "DEBUG" => Level::DEBUG,
                "INFO" => Level::INFO,
                "WARN" => Level::WARN,
                "ERROR" => Level::ERROR,
                _ => Level::TRACE,
            };

            let module = caps["module"].to_string();
            let file = format!("{}:{}", &caps["file"], &caps["line"]);
            let message = caps["message"].to_string();

            LogEntry {
                timestamp,
                level,
                target: module.clone(),
                module,
                file,
                message,
            }
        } else {
            panic!("Failed to parse log entry");
        }
    }
}
