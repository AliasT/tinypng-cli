use base64::{decode, encode};
use reqwest;
use std::cell::RefCell;
use std::convert::AsRef;
use std::env;
use std::fs::{self, DirEntry, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process;
use tokio::{task};
use std::future::{Future, Ready};
use tokio::runtime::Runtime;

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
    let tiny_png = TinyPNG {
        authorization: &authorization,
        client,
        handles: Vec::new(),
    };
    tiny_png.walk(&Path::new(&target));
    tiny_png.wait().await;
}

struct TinyPNG<'a> {
    authorization: &'a str,
    client: reqwest::Client,

    handles: Vec<tokio::task::JoinHandle<()>>,
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

impl<'a> TinyPNG<'a> {
    pub async fn post_file(&self, file_path: &Path) -> Result<(), TinyPNGError> {
        let f = File::open(file_path).unwrap();
        let mut reader = BufReader::new(f);
        reader.fill_buf();

        let req = self
            .client
            .post(TargetURL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth("api", Some(self.authorization))
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

    pub async fn wait(&self) {
        // for h in self.handles {
          //  h.await;
        //}
    }

    pub fn walk(self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.walk(&path)?;
                } else {
                    self.handles.push(task::spawn(async {
                        self.post_file(&path).await;
                    }));
                }
            }
        }
        Ok(())
    }
}
