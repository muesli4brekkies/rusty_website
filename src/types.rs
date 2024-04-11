use std::{io, net, time};

pub type GenFold<'g> = Box<dyn FnMut(String, &SpecInfo) -> String + 'g>;

pub type SpecFold<'s> = Box<dyn FnMut(String, usize) -> String + 's>;

pub type Condition = Box<dyn Fn(&(usize, &String)) -> bool>;

pub type Buffer<'b> = io::BufReader<&'b mut net::TcpStream>;

pub type Categories = Vec<CatInfo>;

pub type Genera = Vec<GenInfo>;

pub type Species = Vec<SpecInfo>;

pub type YamlChunks = Vec<Vec<String>>;

pub type CxnLog<'l> = &'l mut String;

pub type Content = Vec<u8>;

pub type Request = Vec<String>;

pub struct RequestInfo {
    pub host: Option<Host>,
    pub path: Option<String>,
    pub ip: Option<String>,
    pub referer: Option<String>,
}

pub struct ReqFields {
    pub ip: &'static str,
    pub referer: &'static str,
}

#[derive(Clone, Copy)]
pub enum LogFmt {
    // Start
    Timestamp,
    // Request
    Ip,
    Referer,
    Path,
    Host,
    // Response
    Status,
    Length,
    Turnaround,
    // Info
    Uptime,
    NumCon,
}

pub struct RequestLog {
    pub ip: (LogFmt, Option<String>),
    pub host: (LogFmt, Option<Host>),
    pub referer: (LogFmt, Option<String>),
    pub path: (LogFmt, Option<String>),
}

pub struct ResponseLog {
    pub status: (LogFmt, String),
    pub length: (LogFmt, usize),
    pub turnaround: (LogFmt, time::SystemTime),
}

pub struct InfoLog {
    pub uptime: (LogFmt, time::SystemTime),
    pub num_con: (LogFmt, u64),
}

pub struct Log {
    pub request: RequestLog,
    pub response: ResponseLog,
    pub info: InfoLog,
}

pub enum Host {
    Site,
    Mycology,
}

pub enum Layer {
    Category,
    Genus,
    Species,
}

pub struct Response {
    pub status: &'static str,
    pub mime_type: &'static str,
    pub content: Vec<u8>,
}

pub struct Paths {
    pub root: &'static str,
    pub nf404: &'static str,
    pub pd403: &'static str,
    pub meta: &'static str,
    pub menu: &'static str,
    pub shroompage: &'static str,
}

pub struct Templates {
    pub nf404: String,
    pub pd403: String,
    pub menu: String,
    pub myc_page: String,
}

pub struct CatInfo {
    pub title: String,
    pub label: String,
    pub genera: Vec<GenInfo>,
}

pub struct GenInfo {
    pub title: String,
    pub species: Vec<SpecInfo>,
}

pub struct SpecInfo {
    pub title: String,
    pub name: String,
    pub blurb: String,
}
