use {
    crate::{
        consts::{self, domains},
        types::{CxnLog, Host, InfoLog, Log, LogFmt, RequestLog, ResponseLog},
    },
    std::{array, fs, io::Write, time},
};

pub trait Err {
    fn log_err(self);
}

impl Err for String {
    fn log_err(self) {
        let err = format!("ERROR - {}\n", self);
        eprint!("{err}");
        flush(err);
    }
}

fn write_log(string: String, log_type: LogFmt, cxn_log: CxnLog) {
    let line = match log_type {
        LogFmt::Timestamp => format!("\tDate: {string}\n"),

        LogFmt::Path => format!("\tRequest:\n\t\tPath: {string}\n"),
        LogFmt::Host => format!("\t\tHost: {string}\n"),
        LogFmt::Referer => format!("\t\tReferer: {string}\n"),
        LogFmt::Ip => format!("\t\tIp: {string}\n"),

        LogFmt::Status => format!("\tResponse:\n\t\tStatus: {string}\n"),
        LogFmt::Length => format!("\t\tLength: {string} bytes\n"),
        LogFmt::Turnaround => format!("\t\tTurnaround: {string}\n"),

        LogFmt::Uptime => format!("\tUp-time:{string}\n"),
        LogFmt::NumCon => format!("END connection {string}\n"),
    };
    cxn_log.push_str(&line);
}

pub trait Destructure {
    fn destructure(self) -> array::IntoIter<(LogFmt, String), 9>;
}

impl Destructure for Log {
    fn destructure(self) -> array::IntoIter<(LogFmt, String), 9> {
        let Log {
            request:
                RequestLog {
                    ip: (ip_fmt, ip),
                    host: (host_fmt, host),
                    referer: (referer_fmt, referer),
                    path: (path_fmt, path),
                },
            response:
                ResponseLog {
                    status: (status_fmt, status),
                    length: (length_fmt, length),
                    turnaround: (turnaround_fmt, turnaround),
                },
            info:
                InfoLog {
                    uptime: (uptime_fmt, uptime),
                    num_con: (num_con_fmt, num_con),
                },
        } = self;
        [
            (path_fmt, path.unwrap_or("None".to_string())),
            (host_fmt, host.to_string()),
            (ip_fmt, ip.unwrap_or("None".to_string())),
            (referer_fmt, referer.unwrap_or("None".to_string())),
            (status_fmt, status),
            (length_fmt, length.to_string()),
            (turnaround_fmt, turnaround.to_elapsed()),
            (uptime_fmt, uptime.to_uptime()),
            (num_con_fmt, num_con.to_string()),
        ]
        .into_iter()
    }
}

pub trait Logging {
    fn log_this(self, cxn_log: CxnLog);
}

impl Logging for Log {
    fn log_this(self, cxn_log: CxnLog) {
        self.destructure().for_each(|(log_type, log)| {
            write_log(log.to_string(), log_type, cxn_log);
        });
    }
}

trait ToString {
    fn to_string(self) -> String;
}

impl ToString for Option<Host> {
    fn to_string(self) -> String {
        match self {
            Some(v) => match v {
                Host::Mycology => domains::MYCOLOGY.to_string(),
                Host::Site => domains::NO_DOMAIN.to_string(),
            },
            None => "None".to_string(),
        }
    }
}

impl Logging for time::SystemTime {
    fn log_this(self, cxn_log: CxnLog) {
        write_log(self.to_timestamp(), LogFmt::Timestamp, cxn_log);
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

trait TimeManip {
    fn to_elapsed(self) -> String;
    fn to_uptime(self) -> String;
}

impl TimeManip for time::SystemTime {
    fn to_elapsed(self) -> String {
        let time = self.elapsed().unwrap().as_micros();
        if time < 1e3 as u128 {
            format!("{}μs", time)
        } else if time < 1e6 as u128 {
            format!("{}ms", time / 1e3 as u128)
        } else {
            format!("{}s", time / 1e6 as u128)
        }
    }

    fn to_uptime(self) -> String {
        self.elapsed().unwrap().as_secs().to_wdhms()
    }
}

trait ToWdhms {
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

pub fn flush(log: String) {
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
