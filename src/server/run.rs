use {
  crate::{
    consts,
    log::{self, Log},
    mycology::{self, parse},
    server::{
      request::*,
      response::{self, *},
    },
    types::{Categories, Content, IpAddr, Result},
  },
  std::{sync::Arc, time::SystemTime},
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
  let mut last_modified = SystemTime::UNIX_EPOCH;
  let mut yaml: Arc<Categories> = Arc::new(vec![]); // empty vec to initialise
  let cxn_info = Arc::new(Mutex::new(CxnInfo {
    ip: [0, 0, 0, 0],
    unique_cxn: 1,
    total_cxn: 1,
  }));

  let log_file = log::open().await;

  let uptime = SystemTime::now();

  let listener = TcpListener::bind("127.0.0.1:7878").await?;

  loop {
    let (stream, _) = listener.accept().await?;

    (yaml, last_modified) = memo_yaml(last_modified, yaml).await?;

    let (log_file, yaml, cxn_info) = (log_file.try_clone().await?, yaml.clone(), cxn_info.clone());

    tokio::spawn(async move {
      if let Err(e) = handle_connection(stream, uptime, &yaml, log_file, cxn_info).await {
        eprintln!("{}", e)
      }
    });
  }
}

async fn handle_connection(
  mut stream: TcpStream,
  uptime: SystemTime,
  yaml: &Categories,
  log_file: fs::File,
  cxn_info: Arc<Mutex<CxnInfo>>,
) -> Result<()> {
  let cxn_time = SystemTime::now();

  let RequestInfo {
    host,
    path,
    user_agent,
    ip,
    referer,
  } = parse(BufReader::new(&mut stream)).await?;

  let mut cxn_info = cxn_info.lock().await;
  let (unique_cxn, total_cxn, last_ip) = (cxn_info.unique_cxn, cxn_info.total_cxn, cxn_info.ip);

  if ip.unwrap_or_default() != last_ip {
    cxn_info.unique_cxn += 1;
  }
  cxn_info.total_cxn += 1;
  cxn_info.ip = ip.unwrap_or_default();
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
    .fold(String::new(), |a, b| match b.contains("HTTP") {
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

async fn memo_yaml(
  last_modified: SystemTime,
  memo: Arc<Categories>,
) -> Result<(Arc<Categories>, SystemTime)> {
  let curr_modified = fs::metadata(consts::YAML_FILE).await?.modified()?;
  if last_modified == curr_modified {
    Ok((memo, curr_modified))
  } else {
    println!("*** YAML CHANGE DETECTED - RELOADED ***");
    Ok((Arc::new(parse::yaml().await), curr_modified))
  }
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
