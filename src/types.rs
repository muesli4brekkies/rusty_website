use {
  crate::mycology::generate::{CatInfo, GenInfo, SpecInfo},
  std::{io, net},
};

pub type GenFold<'g> = Box<dyn FnMut(String, &SpecInfo) -> String + 'g>;

pub type SpecFold<'s> = Box<dyn FnMut(String, usize) -> String + 's>;

pub type Condition = Box<dyn Fn(&(usize, &String)) -> bool>;

pub type Buffer<'b> = io::BufReader<&'b mut net::TcpStream>;

pub type Categories = Vec<CatInfo>;

pub type Genera = Vec<GenInfo>;

pub type Species = Vec<SpecInfo>;

pub type YamlChunks = Vec<Vec<String>>;

pub type CxnLog<'l> = &'l mut String;

pub type IpAddr = [u8; 4];

pub type Content = Vec<u8>;

pub type Request = Vec<String>;

pub mod tubes {
  use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
  };
  pub type Tubes<T> = (Arc<Mutex<Sender<T>>>, Arc<Mutex<Receiver<T>>>);

  pub type RecvTube<T> = Arc<Mutex<Receiver<T>>>;

  pub type SendTube<T> = Arc<Mutex<Sender<T>>>;
}
