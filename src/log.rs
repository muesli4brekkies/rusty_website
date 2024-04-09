use crate::{consts, types};
use std::{fmt, fs, io, time};

pub trait Err {
    fn log_err(self);
}

impl Err for String {
    fn log_err(self) {
        let err = format!("ERROR - {}\n", self);
        eprint!("{err}");
        write_log(err);
    }
}

use types::{CxnLog, LogKind, Request};
pub trait Logging {
    fn tee_to_log(self, log_type: LogKind, cxn_log: CxnLog) -> Self;
}

impl Logging for Request {
    fn tee_to_log(self, log_type: LogKind, cxn_log: CxnLog) -> Self {
        "\tRequest:\n".tee_to_log(LogKind::NoFmt, cxn_log);
        Request(
            self.0
                .into_iter()
                .map(|l| l.tee_to_log(log_type, cxn_log))
                .collect(),
        )
    }
}

impl<T> Logging for T
where
    T: fmt::Display,
{
    fn tee_to_log(self, log_type: LogKind, cxn_log: CxnLog) -> Self {
        let line = match log_type {
            LogKind::Request => format!("\t\t{self}\n"),
            LogKind::Length => format!("\t\tLength: {self} bytes\n"),
            LogKind::Status => format!("\tResponse:\n\t\tStatus: {self}\n"),

            LogKind::Elapsed => format!("\t\tTime: {self}μs \n"),
            LogKind::Uptime => format!("\tUp-time:{self}\n"),
            LogKind::Timestamp => format!("\tDate: {self}\n"),

            LogKind::End => format!("END connection {self}\n"),

            LogKind::NoFmt => self.to_string(),
        };
        cxn_log.push_str(&line);
        self
    }
}

pub trait ToTimeStamp {
    fn to_timestamp(self) -> String;
}

impl ToTimeStamp for time::SystemTime {
    fn to_timestamp(self) -> String {
        humantime::format_rfc3339_millis(self)
            .to_string()
            .replace('T', "\n\tTime: ")
            .replace('Z', "")
    }
}

use types::Log;
pub fn end<T: fmt::Display>(log_vec: Vec<Log<T>>, cxn_log: CxnLog) {
    vec![
        LogKind::Status,
        LogKind::Length,
        LogKind::Elapsed,
        LogKind::Uptime,
        LogKind::End,
    ]
    .into_iter()
    .zip(log_vec)
    .for_each(|(log_type, log)| {
        log.0.tee_to_log(log_type, cxn_log);
    });
}

use io::Write;
pub fn write_log(log: String) {
    match fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(consts::LOG_FILE)
    {
        Err(e) => eprintln!("{} {} - cannot open log file", e, consts::LOG_FILE),
        Ok(mut v) => {
            if let Err(e) = v.write_all(log.as_bytes()) {
                eprintln!("{} {} - error writing to log file", e, consts::LOG_FILE)
            }
        }
    }
}

pub trait ToWdhms {
    fn to_wdhms(self) -> String;
}

impl ToWdhms for u64 {
    fn to_wdhms(self) -> String {
        [
            ("weeks", self / 604800),
            ("days", (self / 86400) % 7),
            ("hours", (self / 3600) % 24),
            ("mins", (self / 60) % 60),
            ("secs", self % 60),
        ]
        .into_iter()
        .fold("".to_string(), |a, (b, time)| {
            if time != 0 {
                format!("{a} {time} {b}")
            } else {
                a
            }
        })
    }
}
