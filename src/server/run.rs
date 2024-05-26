use {
  crate::{
    consts,
    log::{self, *},
    mycology::{self, parse},
    server::{
      request::*,
      response::{self, *},
    },
    types::{Categories, Content, IpAddr, Result},
  },
  std::{sync::Arc, time},
  tokio::{
    fs,
    io::{AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
  },
};

struct CxnInfo {
  ip: IpAddr,
  unique_cxn: u64,
  total_cxn: u64,
}

pub async fn start_server() -> Result<()> {
  let mut last_mod = time::SystemTime::UNIX_EPOCH;
  let mut yaml: Categories = vec![];
  let cxn_info = Arc::new(Mutex::new(CxnInfo {
    ip: [0, 0, 0, 0],
    unique_cxn: 0,
    total_cxn: 0,
  }));

  let log_file = fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open(consts::LOG_FILE)
    .await
    .unwrap_or_else(|_| panic!("{} - cannot open log file", consts::LOG_FILE));

  let uptime = time::SystemTime::now();

  let listener = TcpListener::bind("127.0.0.1:7878").await?;

  loop {
    let (stream, _) = listener.accept().await?;
    let log_file = log_file.try_clone().await?;
    let curr_mod = fs::metadata(consts::YAML_FILE).await?.modified()?;

    (yaml, last_mod) = if last_mod != curr_mod {
      println!("*** YAML CHANGE DETECTED - RELOADED ***");
      (parse::yaml().await, curr_mod)
    } else {
      (yaml, curr_mod)
    };
    let yaml = yaml.clone();
    let cxn_info = cxn_info.clone();

    tokio::spawn(async move {
      if let Err(e) = handle_connection(stream, uptime, log_file, yaml, cxn_info).await {
        eprintln!("{}", e)
      }
    });
  }
}

async fn handle_connection(
  mut stream: TcpStream,
  uptime: time::SystemTime,
  log_file: fs::File,
  yaml: Categories,
  cxn_info: Arc<Mutex<CxnInfo>>,
) -> Result<()> {
  let cxn_time = time::SystemTime::now();

  let RequestInfo {
    host,
    path,
    user_agent,
    ip,
    referer,
  } = parse(BufReader::new(&mut stream)).await?;

  let mut cxn_info = cxn_info.lock().await;
  let unique_cxn = cxn_info.unique_cxn;
  let total_cxn = cxn_info.total_cxn;
  let last_ip = cxn_info.ip;
  cxn_info.ip = ip.unwrap_or_default();
  if ip.unwrap_or_default() != last_ip {
    cxn_info.unique_cxn += 1;
  }
  cxn_info.total_cxn += 1;
  drop(cxn_info);

  let response = if let (Some(domain), Some(path)) = (&host, &path) {
    match domain {
      Host::Mycology => mycology::generate::get(yaml, path).await,
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

  log::this(
    Log {
      path,
      host,
      ip,
      user_agent,
      referer,
      status,
      length,
      cxn_time,
      start_time: uptime,
      unique_cxn,
      total_cxn,
    }
    .stringify(ip.unwrap_or_default(), last_ip),
    log_file,
  )
  .await;
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
