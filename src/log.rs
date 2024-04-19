use {
    crate::{
        consts::{self, domains},
        server::response::Host,
        types::{CxnLog, IpAddr},
    },
    std::{fmt, fs, io::Write, thread, time},
};

pub struct Log {
    pub path: Option<String>,
    pub host: Option<Host>,
    pub user_agent: Option<String>,
    pub ip: Option<IpAddr>,
    pub referer: Option<String>,
    pub status: String,
    pub length: usize,
    pub thread: usize,
    pub cxn_time: time::SystemTime,
    pub start_time: time::SystemTime,
    pub tally: Tally,
}

#[derive(Clone, Copy)]
pub struct Tally {
    pub unique_conn: u64,
    pub total_conn: u64,
}

pub trait Err {
    fn log_err(self);
}

impl<T> Err for T
where
    T: fmt::Debug,
{
    fn log_err(self) {
        let err = format!("ERROR - {:?}\n", self);
        eprint!("{err}");
        flush(err);
    }
}
pub trait Logging {
    fn log_this(self, cxn_log: CxnLog, is_same_ip: bool);
}

impl Logging for Log {
    fn log_this(self, cxn_log: CxnLog, is_same_ip: bool) {
        let Log {
            path,
            host,
            user_agent,
            ip,
            referer,
            status,
            length,
            thread,
            cxn_time,
            start_time,
            tally: Tally {
                unique_conn,
                total_conn,
            },
        } = self;

        let thread = thread + 1;
        let ip = ip.to_string();
        let path = path.unwrap_or("None".to_string());
        let timestamp = cxn_time.to_timestamp();
        let uptime = start_time.to_uptime();
        let host = host.to_string();
        let referer = referer.unwrap_or("None".to_string());
        let user_agent = user_agent.unwrap_or("None".to_string());
        let turnaround = cxn_time.to_elapsed();
        let tot_threads = thread::available_parallelism().unwrap().get();

        let string = if is_same_ip {
            format!(
        "#{total_conn} - t{thread} - {ip} - {timestamp} - {status} - {length}b - {turnaround} - {path}\n",
      )
        } else {
            format!(
                "START
Timestamp: {timestamp}
Thread: {thread}/{tot_threads}
# Unique: {unique_conn}
# Total: {total_conn}
Up-time:{uptime}
Request:
\tPath: {path}
\tHost: {host}
\tIp: {ip}
\tReferer: {referer}
\tAgent: {user_agent}
Response:
\tStatus: {status}
\tLength: {length} bytes
\tTurnaround: {turnaround}\n"
            )
        };
        cxn_log.push_str(&string);
    }
}

trait ToString {
    fn to_string(self) -> String;
}

impl ToString for Option<IpAddr> {
    fn to_string(self) -> String {
        match self {
            Some(v) => {
                format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3],)
            }
            None => "No IP".to_string(),
        }
    }
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

pub trait ToTimeStamp {
    fn to_timestamp(self) -> String;
}

impl ToTimeStamp for time::SystemTime {
    fn to_timestamp(self) -> String {
        humantime::format_rfc3339_millis(self)
            .to_string()
            .replace('T', " ~ ")
            .replace('Z', "")
    }
}

trait TimeManip {
    fn to_elapsed(self) -> String;
    fn to_uptime(self) -> String;
}

impl TimeManip for time::SystemTime {
    fn to_elapsed(self) -> String {
        || -> Option<String> {
            Some(match self.elapsed().ok()?.as_micros() {
                t if t < 1000 => format!("{}μs", t),
                t if t < 1000000 => format!("{}ms", t / 1000),
                t => format!("{}s", t / 1000000),
            })
        }()
        .unwrap_or("Time has gone backwards :(".to_string())
    }

    fn to_uptime(self) -> String {
        || -> Option<String> { Some(self.elapsed().ok()?.as_secs().to_wdhms()) }()
            .unwrap_or("Time has gone backwards :(".to_string())
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
        .fold("".to_string(), |a, (b, time)| match time {
            0 => a,
            _ => format!("{a} {time} {b}"),
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
