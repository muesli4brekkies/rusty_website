use {
  crate::{
    log::{self, Err, InfoLog, Log, Logging, MiniLog, RequestLog, ResponseLog, Tally},
    mycology,
    server::{
      self,
      request::{Parse, RequestInfo},
      response::{self, CheckErr, Host, Response},
    },
    types::{Content, IpAddr},
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

#[derive(Clone, Copy)]
struct LastConn {
  tally: Tally,
  last_ip: IpAddr,
}

pub fn start_server() {
  let uptime = time::SystemTime::now();
  let listener = net::TcpListener::bind("127.0.0.1:7878").unwrap();
  let mut last_conn: LastConn = LastConn {
    tally: Tally {
      unique_conn: 0,
      total_conn: 0,
    },
    last_ip: [0, 0, 0, 0],
  };
  let templates = server::html::cache();

  listener.incoming().for_each(|stream| {
    last_conn = match handle_connection(stream.unwrap(), uptime, last_conn, &templates) {
      Ok(v) => v,
      Err(e) => {
        e.to_string().log_err();
        last_conn
      }
    }
  });
}

fn handle_connection(
  mut stream: net::TcpStream,
  uptime: time::SystemTime,
  last_conn: LastConn,
  templates: &Templates,
) -> Result<LastConn, io::Error> {
  let LastConn { tally, last_ip } = last_conn;
  let mut cxn_log = String::new();

  let start_time = time::SystemTime::now();

  let request_info: RequestInfo = io::BufReader::new(&mut stream).parse();

  let RequestInfo {
    host,
    path,
    user_agent,
    ip,
    referer,
  } = request_info;

  let this_ip = ip.unwrap_or_default();

  let tally: Tally = Tally {
    total_conn: tally.total_conn + 1,
    unique_conn: if this_ip != last_ip {
      tally.unique_conn + 1
    } else {
      tally.unique_conn + 0
    },
  };

  let response = if let (Some(domain), Some(path)) = (&host, &path) {
    match domain {
      Host::Mycology => mycology::generate::get(path, templates),
      Host::Site => server::response::get(path).check_err(templates),
    }
  } else {
    response::nf404(templates)
  };

  let status = response.status.split_whitespace().collect::<Vec<&str>>()[1..].join(" ");
  let length = response.content.len();

  stream.write_all(&response.prepend_headers())?;

  if this_ip != last_ip {
    Log {
      request: RequestLog {
        start_time,
        path,
        host,
        ip,
        user_agent,
        referer,
      },
      response: ResponseLog {
        status,
        length,
        turnaround: start_time,
      },
      info: InfoLog { uptime, tally },
    }
    .log_this(&mut cxn_log);
  } else {
    MiniLog {
      total_conn: tally.total_conn,
      path,
      turnaround: start_time,
      length,
    }
    .log_this(&mut cxn_log);
  }

  println!("{}", &cxn_log);
  log::flush(cxn_log);

  Ok(LastConn {
    tally,
    last_ip: this_ip,
  })
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
