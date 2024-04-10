use {
  crate::{
    consts::{domains, FIELDS},
    types::{Buffer, Host, Request, RequestInfo},
  },
  std::io::BufRead,
};

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
      host: request.get_domain(),
      path: request.get_path(),
      ip: request.get_field(FIELDS.ip),
      referer: request.get_field(FIELDS.referer),
    }
  }
}

pub trait GetInfo {
  fn get_path(&self) -> Option<String>;
  fn get_domain(&self) -> Option<Host>;
  fn get_field(&self, field: &'static str) -> Option<String>;
}

impl GetInfo for Request {
  fn get_path(&self) -> Option<String> {
    Some(self.first()?.split_whitespace().nth(1)?.to_string())
  }

  fn get_domain(&self) -> Option<Host> {
    match self.iter().find(|l| l.starts_with("Host")) {
      Some(v) => match v.replace("Host: ", "").as_str() {
        domains::MYCOLOGY => Some(Host::Mycology),
        domains::NO_DOMAIN => Some(Host::Site),
        _ => None,
      },
      None => None,
    }
  }

  fn get_field(&self, field: &'static str) -> Option<String> {
    self
      .iter()
      .find(|l| l.starts_with(field))
      .map(|v| v.replace(field, ""))
  }
}
