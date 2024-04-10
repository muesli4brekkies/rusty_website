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
  fn get_domain(&self) -> Option<String>;
  fn get_path(&self) -> Option<String>;
}

impl GetReqInfo for Request {
  fn get_domain(&self) -> Option<String> {
    if let Some(v) = self.0.get(1) {
      let host = v.split('.').next().unwrap().replace("Host: ", "");
      if host == consts::HOST_NAME {
        Some("".to_string())
      } else {
        Some(host)
      }
    } else {
      None
    }
  }

  fn get_path(&self) -> Option<String> {
    Some(self.0.first()?.split_whitespace().nth(1)?.to_string())
  }
}

use {consts::domains, types::Domain};
pub fn verify_domain(requested_domain: Option<String>) -> Option<Domain> {
  match requested_domain {
    Some(v) => match v.as_str() {
      domains::NO_DOMAIN => Some(Domain::Site),
      domains::MYCOLOGY => Some(Domain::Mycology),
      _ => None,
    },
    None => None,
  }
}
