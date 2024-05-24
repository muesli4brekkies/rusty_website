use {
  crate::{
    log::*,
    mycology,
    server::{
      request::*,
      response::{self, *},
    },
    types::{tubes::*, Content, Result},
  },
  std::{
    sync::{mpsc, Arc, Mutex},
    thread, time,
  },
  tokio::{
    io::{AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
  },
};

pub async fn start_server() -> Result<()> {
  let (log_send, log_recv) = make_tube();

  let uptime = time::SystemTime::now();

  thread::spawn(|| logger(log_recv));

  let listener = TcpListener::bind("127.0.0.1:7878").await?;

  loop {
    let (stream, _) = listener.accept().await?;
    let log_send = log_send.clone();

    tokio::spawn(async move {
      if let Err(e) = handle_connection(stream, uptime, log_send).await {
        eprintln!("{}", e.to_string())
      }
    });
  }
}

async fn handle_connection(
  mut stream: TcpStream,
  uptime: time::SystemTime,
  log_send: SendTube<Log>,
) -> Result<()> {
  let cxn_time = time::SystemTime::now();

  let RequestInfo {
    host,
    path,
    user_agent,
    ip,
    referer,
  } = parse(BufReader::new(&mut stream)).await?;

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

  stream.write_all(&response.prepend_headers()).await?;
  stream.flush().await?;

  log_send
    .lock()
    .unwrap()
    .send(Log {
      path,
      host,
      ip,
      user_agent,
      referer,
      status,
      length,
      cxn_time,
      start_time: uptime,
    })
    .unwrap();
  Ok(())
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
      .into_bytes(),
      self.content,
    ]
    .concat()
  }
}

pub fn make_tube<T>() -> Tubes<T> {
  let (r, s) = mpsc::channel();
  (Arc::new(Mutex::new(r)), Arc::new(Mutex::new(s)))
}
