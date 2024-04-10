use crate::types::RequestInfo;
use crate::{consts, types};
use std::io;

use types::Buffer;
pub trait Parse {
  fn to_lines(self) -> Request;
}

impl Parse for Buffer<'_> {
  fn to_lines(self) -> Request {
    use io::BufRead;
    self
      .lines()
      .map(Result::unwrap)
      .take_while(|l| !l.is_empty())
      .collect::<Vec<String>>()
      .wrap()
  }
}

use types::Request;
pub trait WrapRequest {
  fn wrap(self) -> Request;
}

impl WrapRequest for Vec<String> {
  fn wrap(self) -> Request {
    Request(self)
  }
}

pub trait GetReqInfo {
  fn get_info(&self) -> RequestInfo;

  fn get_path(&self) -> Option<String>;
  fn get_domain(&self) -> Option<Host>;
  fn get_field(&self, field: &'static str) -> Option<String>;
}

use {
  consts::{domains, FIELDS},
  types::Host,
};
impl GetReqInfo for Request {
  fn get_info(&self) -> RequestInfo {
    RequestInfo {
      host: self.get_domain(),
      path: self.get_path(),
      ip: self.get_field(FIELDS.ip),
      referer: self.get_field(FIELDS.referer),
    }
  }

  fn get_path(&self) -> Option<String> {
    Some(self.0.first()?.split_whitespace().nth(1)?.to_string())
  }

  fn get_domain(&self) -> Option<Host> {
    match self.0.iter().find(|l| l.starts_with("Host")) {
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
      .0
      .iter()
      .find(|l| l.starts_with(field))
      .map(|v| v.replace(field, ""))
  }
}
