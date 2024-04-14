use {
  crate::{
    consts::{domains, FIELDS},
    server::response::Host,
    types::{Buffer, IpAddr, Request},
  },
  std::io::BufRead,
};

pub struct RequestInfo {
  pub host: Option<Host>,
  pub path: Option<String>,
  pub user_agent: Option<String>,
  pub ip: Option<IpAddr>,
  pub referer: Option<String>,
}

pub trait Parse {
  fn parse(self) -> RequestInfo;
}

impl Parse for Buffer<'_> {
  fn parse(self) -> RequestInfo {
    let request = self
      .lines()
      .map(Result::unwrap)
      .take_while(|l| !l.is_empty())
      .collect::<Vec<String>>();
    RequestInfo {
      host: request.get_host(),
      path: request.get_path(),
      user_agent: request.get_field(FIELDS.user_agent),
      ip: request.get_ip(),
      referer: request.get_field(FIELDS.referer),
    }
  }
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
      .and_then(|v| vec_u8_from_ip(v))
  }

  fn get_field(&self, field: &'static str) -> Option<String> {
    self
      .iter()
      .find(|l| l.starts_with(field))
      .map(|v| v.replace(field, ""))
  }
}

fn vec_u8_from_ip(ip: &str) -> Option<[u8; 4]> {
  let ip_vec: Vec<Option<u8>> = ip
    .replace(FIELDS.ip, "")
    .split('.')
    .map(|n| n.parse::<u8>().ok())
    .collect();
  ip_vec
    .iter()
    .all(|o| o.is_some())
    .then(|| {
      ip_vec
        .into_iter()
        .map(Option::unwrap)
        .collect::<Vec<u8>>()
        .try_into()
        .ok()
    })
    .flatten()
}
