use {
    crate::{
        consts::{self, domains},
        server::response::Host,
        types::IpAddr,
    },
    std::{fmt, time},
    tokio::{fs, io::AsyncWriteExt},
};

pub struct Log {
    pub path: Option<String>,
    pub host: Option<Host>,
    pub user_agent: Option<String>,
    pub ip: Option<IpAddr>,
    pub referer: Option<String>,
    pub status: String,
    pub length: usize,
    pub cxn_time: time::SystemTime,
    pub start_time: time::SystemTime,
}

impl Log {
    pub fn stringify(self) -> String {
        let none = ||"None".to_owned();
        let unique_conn = 0;
        let total_conn = 0;
        let Log {
            path,
            host,
            user_agent,
            ip,
            referer,
            status,
            length,
            cxn_time,
            start_time,
        } = self;

        let ip_str = ip.to_string();
        let path = path.unwrap_or_else(none);
        let timestamp = cxn_time.to_string();
        let uptime = start_time.to_uptime();
        let host = host.to_string();
        let referer = referer.unwrap_or_else(none);
        let user_agent = user_agent.unwrap_or_else(none);
        let turnaround = cxn_time.to_elapsed();

        let mini_log = |total_conn: i32| {
            format!(
        "#{total_conn} - {ip_str} - {timestamp} - {status} - {length}b - {turnaround} - {path}\n"
      )
        };

        let big_log = |total_conn: i32, unique_conn: i32| {
            format!(
                "START\n\
            Timestamp: {timestamp}\n\
            # Unique: {unique_conn}\n\
            # Total: {total_conn}\n\
            Up-time:{uptime}\n\
            Request:\n\
            \tPath: {path}\n\
            \tHost: {host}\n\
            \tIp: {ip_str}\n\
            \tReferer: {referer}\n\
            \tAgent: {user_agent}\n\
            Response:\n\
            \tStatus: {status}\n\
            \tLength: {length} bytes\n\
            \tTurnaround: {turnaround}\n"
            )
        };
        //    mini_log(total_conn)
        big_log(total_conn, unique_conn)
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

impl ToString for time::SystemTime {
    fn to_string(self) -> String {
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
        || -> Result<String, time::SystemTimeError> {
            Ok(match self.elapsed()?.as_micros() {
                t if t < 1000 => format!("{}μs", t),
                t if t < 1000000 => format!("{}ms", t / 1000),
                t => format!("{}s", t / 1000000),
            })
        }()
        .unwrap_or("Time has gone backwards :(".to_string())
    }

    fn to_uptime(self) -> String {
        || -> Result<String, time::SystemTimeError> { Ok(self.elapsed()?.as_secs().to_wdhms()) }()
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

pub async fn this(string: String, mut log_file: fs::File) {
    if let Err(e) = log_file.write(string.as_bytes()).await {
        eprintln!("{} {} - error writing to log file", e, consts::LOG_FILE)
    }
    print!("{string}");
}
