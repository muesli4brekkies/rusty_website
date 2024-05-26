use {
    crate::{
        consts,
        log::{self, *},
        mycology::{self, parse},
        server::{
            request::*,
            response::{self, *},
        },
        types::{Categories, Content, Result},
    },
    std::time,
    tokio::{
        fs,
        io::{AsyncWriteExt, BufReader},
        net::{TcpListener, TcpStream},
    },
};

pub async fn start_server() -> Result<()> {
    let mut last_mod = time::SystemTime::UNIX_EPOCH;
    let mut yaml:Categories = vec![];

    let log_file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(consts::LOG_FILE)
        .await
        .expect(&format!("{} - cannot open log file", consts::LOG_FILE));

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

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, uptime, log_file, yaml).await {
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
        }
        .stringify(),
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
