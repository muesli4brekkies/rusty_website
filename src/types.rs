use {
    crate::mycology::generate::{CatInfo, SpeInfo},
    std::{error, result},
    tokio::{io::BufReader, net::TcpStream},
};

pub type GenFold<'g> = Box<dyn FnMut(String, &SpeInfo) -> String + 'g>;

pub type SpecFold<'s> = Box<dyn FnMut(String, usize) -> String + 's>;

pub type Condition = Box<dyn Fn(&(usize, &String)) -> bool>;

pub type Buffer<'b> = BufReader<&'b mut TcpStream>;

pub type Categories = Vec<CatInfo>;

pub type YamlString = String;

pub type YamlChunks = Vec<Vec<YamlLine>>;

pub type YamlLines = Vec<YamlLine>;

pub type YamlLine = String;

pub type CxnLog<'l> = &'l mut String;

pub type IpAddr = [u8; 4];

pub type Content = Vec<u8>;

pub type Request = Vec<String>;

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;
