use reqwest;
use std::cell::RefCell;
use std::convert::AsRef;
use std::env;
use std::fs::{self, DirEntry, File};
use std::future::{Future, Ready};
use std::io::{self, BufRead, BufReader};
use std::marker::PhantomData;
use std::path::Path;
use std::process;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::task::{self, JoinHandle};

const TargetURL: &'static str = "https://api.tinify.com/shrink";

#[tokio::main]
async fn main() {
    let authorization = env::var("TINY_PNG_KEY").unwrap_or("".to_string());
    if authorization.len() == 0 {
        eprintln!("please provide a tiny png api key");
        process::exit(1);
    }
    let target = env::args().nth(1).unwrap_or(".".to_string());
    let client = reqwest::Client::new();
    let mut tiny = Tiny {
        inner: TinyPNG {
            authorization,
            client,
        },
    };
    let mut handles = Vec::new();
    // let tiny = tiny.lock().unwrap();
    tiny.walk(&Path::new(&target), &mut handles);

    for h in handles {
        // let h = *h;
        h.await;
    }
}

#[derive(Clone)]
struct TinyPNG {
    authorization: String,
    client: reqwest::Client,
}

struct Tiny {
    inner: TinyPNG,
    // handles: Vec<JoinHandle<()>>,
    // phantom: PhantomData<&'b T>,
}

enum TinyPNGError {
    RequestError(reqwest::Error),
}

// error 转换
impl From<reqwest::Error> for TinyPNGError {
    fn from(err: reqwest::Error) -> Self {
        TinyPNGError::RequestError(err)
    }
}

impl Tiny {
    pub fn walk(&mut self, dir: &Path, handles: &mut Vec<JoinHandle<()>>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.walk(&path, handles)?;
                } else {
                    let tiny_png = self.inner.clone();
                    handles.push(task::spawn(async move {
                        // let c = tiny_png.lock().unwrap();
                        tiny_png.post_file(&path).await;
                    }));
                }
            }
        }
        Ok(())
    }

    pub async fn wait(&self) {}
}

impl TinyPNG {
    pub async fn post_file(&self, file_path: &Path) -> Result<(), TinyPNGError> {
        let req = self
            .client
            .post(TargetURL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth("api", Some(&self.authorization))
            .body(fs::read(file_path).unwrap());

        // self.handles.push(tokio::task::spawn(async move || ->Result<(), TinyPNGError>{
        let text = req.send().await?.text().await?;
        println!("{}", text);
        Ok(())
    }

    // 替换本地文件
    pub fn download_file() {
        //
    }
}
