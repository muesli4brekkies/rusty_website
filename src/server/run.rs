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
    log::{Logging, ToTimeStamp},
    server::{
      request::{GetReqInfo, Parse},
      response::CheckErr,
    },
    types::{EndLog, Host, LogKind, Request},
  };

  let mut cxn_log = format!("\n START connection {num_con}\n");

  let start_cxn = time::SystemTime::now();

  start_cxn
    .to_timestamp()
    .tee_to_log(LogKind::Timestamp, &mut cxn_log);

  let request: Request = io::BufReader::new(&mut stream)
    .to_lines()
    .tee_to_log(LogKind::Request, &mut cxn_log);

  let request_info = request.get_info();

  dbg!(&request_info);
  let (requested_path, requested_domain) = (request_info.path, request_info.host);

  let response = match (requested_domain, requested_path) {
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

  EndLog {
    status,
    length,
    start_cxn,
    start_time,
    num_con,
  }
  .tee_to_log(LogKind::End, &mut cxn_log);

  log::write(cxn_log);

  Ok(num_con)
}

use types::{Content, Response};
trait Prepend {
  fn prepend_headers<'h>(self) -> Content;
}

impl Prepend for Response {
  fn prepend_headers<'h>(self) -> Content {
    vec![
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
