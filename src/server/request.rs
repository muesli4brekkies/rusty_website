use {
  crate::{
    consts::{domains, FIELDS},
    server::response::Host,
    types::{Buffer, IpAddr, Request},
  },
  std::{
    io::{self},
    vec::Vec,
  },
  tokio::io::AsyncBufReadExt,
};

pub struct RequestInfo {
  pub host: Option<Host>,
  pub path: Option<String>,
  pub user_agent: Option<String>,
  pub ip: Option<IpAddr>,
  pub referer: Option<String>,
}

pub async fn parse_tcp_stream(buf: Buffer<'_>) -> Result<RequestInfo, io::Error> {
  let mut lines = buf.lines();
  let mut request = vec![];

  loop {
    if let Some(l) = lines.next_line().await? {
      if l.is_empty() {
        break;
      }
      request.push(l);
    }
  }

  Ok(RequestInfo {
    host: request.get_host(),
    path: request.get_path(),
    user_agent: request.get_field(FIELDS.user_agent),
    ip: request.get_ip(),
    referer: request.get_field(FIELDS.referer),
  })
}

trait GetInfo {
  fn get_path(&self) -> Option<String>;
  fn get_host(&self) -> Option<Host>;
  fn get_ip(&self) -> Option<IpAddr>;
  fn get_field(&self, field: &'static str) -> Option<String>;
}

impl GetInfo for Request {
  fn get_path(&self) -> Option<String> {
    Some(self.first()?.split_whitespace().nth(1)?.to_string())
  }

  fn get_host(&self) -> Option<Host> {
    self.iter().find(|l| l.starts_with("Host")).and_then(|v| {
      match v.replace("Host: ", "").as_str() {
        domains::MYCOLOGY => Some(Host::Mycology),
        domains::NO_DOMAIN => Some(Host::Site),
        _ => None,
      }
    })
  }

  fn get_ip(&self) -> Option<IpAddr> {
    self
      .iter()
      .find(|l| l.starts_with(FIELDS.ip))
      .arr_u8_from_ip()
  }

  fn get_field(&self, field: &'static str) -> Option<String> {
    self
      .iter()
      .find(|l| l.starts_with(field))
      .map(|v| v.replace(field, ""))
  }
}

trait ArrFromIp {
  fn arr_u8_from_ip(self) -> Option<IpAddr>;
}

impl ArrFromIp for Option<&String> {
  fn arr_u8_from_ip(self) -> Option<IpAddr> {
    self.and_then(|v| {
      v.replace(FIELDS.ip, "")
        .split('.')
        .map(|n| n.parse::<u8>().ok())
        .collect::<Option<Vec<u8>>>()
        .and_then(|l| l.try_into().ok())
    })
  }
}
