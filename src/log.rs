use {
  crate::{
    consts::{self, domains},
    server::response::Host,
    types::{CxnLog, IpAddr},
  },
  std::{array, fs, io::Write, time},
};

pub enum LogFmt {
  // Start
  Timestamp,
  // Request
  Path,
  Host,
  UserAgent,
  Ip,
  Referer,
  // Response
  Status,
  Length,
  Turnaround,
  // Info
  Uptime,
  UniqueConn,
  TotalConn,

  Mini,
}

pub struct MiniLog {
  pub total_conn: u64,
  pub path: Option<String>,
  pub turnaround: time::SystemTime,
  pub length: usize,
}

pub struct Log {
  pub request: RequestLog,
  pub response: ResponseLog,
  pub info: InfoLog,
}

pub struct RequestLog {
  pub start_time: time::SystemTime,
  pub path: Option<String>,
  pub host: Option<Host>,
  pub user_agent: Option<String>,
  pub ip: Option<IpAddr>,
  pub referer: Option<String>,
}

pub struct ResponseLog {
  pub status: String,
  pub length: usize,
  pub turnaround: time::SystemTime,
}

pub struct InfoLog {
  pub uptime: time::SystemTime,
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

impl Err for String {
  fn log_err(self) {
    let err = format!("ERROR - {}\n", self);
    eprint!("{err}");
    flush(err);
  }
}

fn write_log(string: String, log_type: LogFmt, cxn_log: CxnLog) {
  let line = match log_type {
    LogFmt::Timestamp => format!("START\n\tDate: {string}\n"),

    LogFmt::Path => format!("\tRequest:\n\t\tPath: {string}\n"),
    LogFmt::Host => format!("\t\tHost: {string}\n"),
    LogFmt::UserAgent => format!("\t\tUser Agent: {string}\n"),
    LogFmt::Referer => format!("\t\tReferer: {string}\n"),
    LogFmt::Ip => format!("\t\tIp: {string}\n"),

    LogFmt::Status => format!("\tResponse:\n\t\tStatus: {string}\n"),
    LogFmt::Length => format!("\t\tLength: {string} bytes\n"),
    LogFmt::Turnaround => format!("\t\tTurnaround: {string}\n"),

    LogFmt::Uptime => format!("\tUp-time:{string}\n"),
    LogFmt::UniqueConn => format!("\tNum Unique Cxns: {string}\n"),
    LogFmt::TotalConn => format!("\tNum Total Cxns: {string}\n"),

    LogFmt::Mini => format!(r#" "" -> {string}"#),
  };
  cxn_log.push_str(&line);
}

pub trait Destructure {
  fn destructure(self) -> array::IntoIter<(LogFmt, String), 12>;
}

impl Destructure for Log {
  fn destructure(self) -> array::IntoIter<(LogFmt, String), 12> {
    let Log {
      request:
        RequestLog {
          start_time,
          ip,
          host,
          user_agent,
          referer,
          path,
        },
      response: ResponseLog {
        status,
        length,
        turnaround,
      },
      info:
        InfoLog {
          uptime,
          tally: Tally {
            unique_conn,
            total_conn,
          },
        },
    } = self;
    [
      (LogFmt::Timestamp, start_time.to_timestamp()),
      (LogFmt::UniqueConn, unique_conn.to_string()),
      (LogFmt::TotalConn, total_conn.to_string()),
      (LogFmt::Uptime, uptime.to_uptime()),
      (LogFmt::Path, path.unwrap_or("None".to_string())),
      (LogFmt::Host, host.to_string()),
      (LogFmt::Ip, ip.to_string()),
      (LogFmt::Referer, referer.unwrap_or("None".to_string())),
      (LogFmt::UserAgent, user_agent.unwrap_or("None".to_string())),
      (LogFmt::Status, status),
      (LogFmt::Length, length.to_string()),
      (LogFmt::Turnaround, turnaround.to_elapsed()),
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
      write_log(log, log_type, cxn_log);
    });
  }
}

impl Logging for MiniLog {
  fn log_this(self, cxn_log: CxnLog) {
    let MiniLog {
      total_conn,
      path,
      turnaround,
      length,
    } = self;
    write_log(
      format!(
        " #{} - {} - {} bytes - {}",
        total_conn,
        turnaround.to_elapsed(),
        length,
        path.unwrap_or("None".to_string()),
      ),
      LogFmt::Mini,
      cxn_log,
    )
  }
}

trait ToString {
  fn to_string(self) -> String;
}

impl ToString for Option<IpAddr> {
  fn to_string(self) -> String {
    match self {
      Some(v) => {
        let mut iter = v.into_iter();
        format!(
          "{}.{}.{}.{}",
          iter.next().unwrap(),
          iter.next().unwrap(),
          iter.next().unwrap(),
          iter.next().unwrap()
        )
      }
      None => "None".to_string(),
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
