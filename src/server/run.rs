use crate::{
    consts, log, mycology,
    server::{html, request, response},
    types::{self, Response},
};
use std::{io, net, time};

pub fn start_server() {
    use net::TcpListener;
    let start_time = time::SystemTime::now();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let mut num_con: u64 = 0;
    let files = html::cache();

    listener.incoming().for_each(|stream| {
        num_con = match handle_connection(stream.unwrap(), start_time, num_con + 1, &files) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
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
    use consts::status;
    use log::{write_log, Logging, ToTimeStamp, ToWdhms};
    use request::{GetReqInfo, Parse, WrapRequest};
    use types::{Domain, Log, LogKind, Request};

    let mut cxn_log = format!("\n START connection {num_con}\n");

    let start_cxn = time::SystemTime::now();

    start_cxn
        .to_timestamp()
        .tee_to_log(LogKind::Timestamp, &mut cxn_log);

    let request: Request = io::BufReader::new(&mut stream)
        .to_lines()
        .wrap()
        .tee_to_log(LogKind::Request, &mut cxn_log);

    let requested_domain = request.get_domain();

    let requested_path = request.get_path().unwrap_or("".to_string());

    dbg!(&requested_domain, &requested_path);
    let response = match response::decide(requested_domain) {
        Some(v) => {
            dbg!(&v);
            match v {
                Domain::Mycology => mycology::mushget::gen_shroom_html(requested_path, templates),
                Domain::Site => match response::get(requested_path) {
                    Ok(v) => v,
                    Err(e) => {
                        dbg!(&e);
                        if e.to_string().contains("Permission denied") {
                            (
                                status::HTTP_403,
                                "text/html",
                                templates.pd403.as_bytes().to_vec(),
                            )
                        } else {
                            (
                                status::HTTP_404,
                                "text/html",
                                templates.nf404.as_bytes().to_vec(),
                            )
                        }
                    }
                },
            }
        }
        None => (
            status::HTTP_404,
            "text/plain",
            "404 lol".as_bytes().to_vec(),
        ),
    };
    let status = response.0.split_whitespace().collect::<Vec<&str>>()[1..].join(" ");
    let length = response.2.len().to_string();
    response.write_to_stream(stream)?;
    let end_log = vec![
        Log(status),
        Log(length),
        Log(start_cxn.elapsed().unwrap().as_micros().to_string()),
        Log(start_time.elapsed().unwrap().as_secs().to_wdhms()),
        Log(num_con.to_string()),
    ];

    log::end(end_log, &mut cxn_log);

    write_log(cxn_log);

    Ok(num_con)
}

trait WriteToStream {
    fn write_to_stream(self, stream: net::TcpStream) -> Result<(), io::Error>;
}

impl WriteToStream for Response {
    fn write_to_stream(self, mut stream: net::TcpStream) -> Result<(), io::Error> {
        use io::prelude::Write;
        let (status, mime_type, mut content) = self;
        let buf = &mut format!(
            "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
            status,
            content.len(),
            mime_type
        )
        .as_bytes()
        .to_vec();
        buf.append(&mut content);
        stream.write_all(buf)
    }
}
