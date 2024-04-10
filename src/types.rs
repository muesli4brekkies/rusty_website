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

#[derive(Debug)]
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

#[derive(Debug)]
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

pub struct Log {
  // Request
  pub ip: Option<String>,
  pub host: Option<Host>,
  pub referer: Option<String>,
  pub path: Option<String>,
  // Response
  pub status: String,
  pub length: usize,
  pub turnaround: time::SystemTime,
  // Data
  pub uptime: time::SystemTime,
  pub num_con: u64,
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
  pub shroompage: String,
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
