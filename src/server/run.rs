use {
  crate::{
    consts::status,
    log::{self, Err, InfoLog, Log, LogFmt, Logging, RequestLog, ResponseLog},
    mycology,
    server::{
      self,
      request::{Parse, RequestInfo},
      response::{CheckErr, Host, Response},
    },
    types::Content,
  },
  std::{
    io::{self, Write},
    net, time,
  },
};

pub struct Templates {
  pub nf404: String,
  pub pd403: String,
  pub menu: String,
  pub myc_page: String,
  pub fragments: Fragments,
}

pub struct Fragments {
  pub category: String,
  pub genus: String,
  pub species: String,
  pub menu: String,
}

pub fn start_server() {
  let start_time = time::SystemTime::now();
  let listener = net::TcpListener::bind("127.0.0.1:7878").unwrap();
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

fn handle_connection(
  mut stream: net::TcpStream,
  start_time: time::SystemTime,
  num_con: u64,
  templates: &Templates,
) -> Result<u64, io::Error> {
  let mut cxn_log = format!("START connection {num_con}\n");

  let start_cxn = time::SystemTime::now();

  start_cxn.log_this(&mut cxn_log);

  let request_info: RequestInfo = io::BufReader::new(&mut stream).parse();

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
    request: RequestLog {
      path: (LogFmt::Path, request_info.path),
      ip: (LogFmt::Ip, request_info.ip),
      host: (LogFmt::Host, request_info.host),
      referer: (LogFmt::Referer, request_info.referer),
    },
    response: ResponseLog {
      status: (LogFmt::Status, status),
      length: (LogFmt::Length, length),
      turnaround: (LogFmt::Turnaround, start_cxn),
    },
    info: InfoLog {
      uptime: (LogFmt::Uptime, start_time),
      num_con: (LogFmt::NumCon, num_con),
    },
  }
  .log_this(&mut cxn_log);

  println!("{}", &cxn_log);
  log::flush(cxn_log);

  Ok(num_con)
}

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
