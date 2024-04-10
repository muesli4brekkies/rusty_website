use crate::log::Logging;
use crate::types::RequestInfo;
use crate::{consts, log, mycology, server, types};
use std::{io, net, time};

use log::Err;
pub fn start_server() {
  use net::TcpListener;
  let start_time = time::SystemTime::now();
  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
  let mut num_con: u64 = 0;
  let files = server::html::cache();

  listener.incoming().for_each(|stream| {
    num_con = match handle_connection(stream.unwrap(), start_time, num_con + 1, &files) {
      Ok(v) => v,
      Err(e) => {
        e.to_string().log_err();
        num_con
      }
    }
  });
}

use types::Templates;
fn handle_connection(
  mut stream: net::TcpStream,
  start_time: time::SystemTime,
  num_con: u64,
  templates: &Templates,
) -> Result<u64, io::Error> {
  use io::Write;
  use {
    consts::status,
    server::{request::Parse, response::CheckErr},
    types::{Host, Log},
  };

  let mut cxn_log = format!("START connection {num_con}\n");

  let start_cxn = time::SystemTime::now();

  start_cxn.log_this(&mut cxn_log);

  let request_info: RequestInfo = io::BufReader::new(&mut stream).get_info();

  let (requested_path, requested_host) = (&request_info.path, &request_info.host);

  let response = match (requested_host, requested_path) {
    (Some(domain), Some(path)) => match domain {
      Host::Mycology => mycology::generate::get(path, templates),
      Host::Site => server::response::get(path).check_err(templates),
    },
    _ => Response {
      status: status::HTTP_404,
      mime_type: "text/plain",
      content: "404 lol".as_bytes().to_vec(),
    },
  };

  let status = response.status.split_whitespace().collect::<Vec<&str>>()[1..].join(" ");
  let length = response.content.len();

  stream.write_all(&response.prepend_headers())?;

  Log {
    path: request_info.path,
    ip: request_info.ip,
    host: request_info.host,
    referer: request_info.referer,
    status,
    length,
    turnaround: start_cxn,
    uptime: start_time,
    num_con,
  }
  .log_this(&mut cxn_log);

  println!("{}", &cxn_log);
  log::flush(cxn_log);

  Ok(num_con)
}

use types::{Content, Response};
trait Prepend {
  fn prepend_headers(self) -> Content;
}

impl Prepend for Response {
  fn prepend_headers(self) -> Content {
    [
      format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
        self.status,
        self.content.len(),
        self.mime_type
      )
      .as_bytes(),
      &self.content,
    ]
    .concat()
  }
}
