use {
  crate::{
    log::{self, *},
    mycology,
    server::{
      request::*,
      response::{self, *},
    },
    thread::*,
    types::{tubes::*, Content, IpAddr},
  },
  std::{
    io::{self, Write},
    net, time,
  },
};

#[derive(Clone, Copy)]
struct LastConn {
  tally: Tally,
  last_ip: IpAddr,
}

pub fn start_server() {
  let (thread_send, thread_recv) = make_tube();
  let pool = ThreadPool::new(thread_send.clone()).unwrap();
  let uptime = time::SystemTime::now();
  let listener = net::TcpListener::bind("127.0.0.1:7878").unwrap();
  let mut last_conn: LastConn = LastConn {
    tally: Tally {
      unique_conn: 0,
      total_conn: 0,
    },
    last_ip: [255, 255, 255, 255],
  };

  listener.incoming().for_each(|stream| {
    let thread_recv = thread_recv.clone();
    let (conn_send, conn_recv) = make_tube();
    pool.execute(move || {
      last_conn = handle_connection(stream.unwrap(), uptime, last_conn, thread_recv).unwrap();
      conn_send.lock().unwrap().send(last_conn).unwrap();
    });
    last_conn = conn_recv.lock().unwrap().recv().unwrap();
  });
}

fn handle_connection(
  mut stream: net::TcpStream,
  uptime: time::SystemTime,
  last_conn: LastConn,
  thread_recv: RecvTube<usize>,
) -> Result<LastConn, io::Error> {
  let thread = thread_recv.lock().unwrap().recv().unwrap();
  let LastConn { tally, last_ip } = last_conn;
  let mut cxn_log = String::new();

  let cxn_time = time::SystemTime::now();

  let request_info: RequestInfo = io::BufReader::new(&mut stream).parse();

  let RequestInfo {
    host,
    path,
    user_agent,
    ip,
    referer,
  } = request_info;

  let is_same_ip = ip.unwrap_or_default() == last_ip;

  let tally: Tally = Tally {
    total_conn: tally.total_conn + 1,
    unique_conn: if is_same_ip {
      tally.unique_conn
    } else {
      tally.unique_conn + 1
    },
  };

  let response = if let (Some(domain), Some(path)) = (&host, &path) {
    match domain {
      Host::Mycology => mycology::generate::get(path),
      Host::Site => response::get(path),
    }
    .replace_err()
  } else {
    err::nf404()
  }?;

  let status = response
    .status
    .split_whitespace()
    .fold("".to_string(), |a, b| match b.contains("HTTP") {
      true => a,
      false => format!("{a} {b}"),
    });
  let length = response.content.len();

  stream.write_all(&response.prepend_headers())?;

  Log {
    request: RequestLog {
      path,
      host,
      ip,
      user_agent,
      referer,
    },
    response: ResponseLog { status, length },
    info: InfoLog {
      thread,
      cxn_time,
      start_time: uptime,
      tally,
    },
  }
  .log_this(&mut cxn_log, is_same_ip);

  print!("{}", &cxn_log);
  log::flush(cxn_log);

  Ok(LastConn {
    tally,
    last_ip: ip.unwrap_or_default(),
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
