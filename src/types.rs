use core::fmt;
use std::{io::BufReader, net::TcpStream};

pub type GenFold<'g> = Box<dyn FnMut(String, &SpecInfo) -> String + 'g>;

pub type SpecFold<'s> = Box<dyn FnMut(String, usize) -> String + 's>;

pub type Condition = Box<dyn Fn(&(usize, &String)) -> bool>;

pub type Buffer<'b> = BufReader<&'b mut TcpStream>;

pub type Categories = Vec<CatInfo>;

pub type Genera = Vec<GenInfo>;

pub type Species = Vec<SpecInfo>;

pub type YamlChunks = Vec<Vec<String>>;

pub type CxnLog<'l> = &'l mut String;

pub type Content = Vec<u8>;

#[derive(Clone, Copy)]
pub enum LogKind {
  NoFmt,
  Timestamp,
  Request,
  Uptime,
  Status,
  Length,
  Elapsed,
  End,
}

#[derive(Debug)]
pub enum Domain {
  Site,
  Mycology,
}

pub enum Layer {
  Category,
  Genus,
  Species,
}

#[derive(Debug)]
pub struct Request(pub Vec<String>);

pub struct Log<T: fmt::Display>(pub T);

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
