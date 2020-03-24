use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::ErrorKind;
use std::io::SeekFrom;
use std::thread;

use serde::Deserialize;
use std::sync::mpsc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = &args[1];

    let mut f = match File::open(config) {
        Ok(fp) => fp,
        Err(_) => panic!(""),
    };
    let mut config_content = String::new();
    let len = f.read_to_string(&mut config_content).unwrap();
    if len == 0 {
        panic!("");
    }
    let config: Config = toml::from_str(config_content.as_str()).unwrap();
    let input = config.file.input.clone();
    let output = config.file.output.clone();
    let regex = config.filter.regex.clone();

    // 定义通道
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        filter(tx, config.file.input.as_str(), config.filter.regex.as_str());
    });

    write(rx, output.as_str());
}

// 定义循环读取文件函数
fn filter(tx: mpsc::Sender<String>, filePath: &str, regex: &str) {
    let mut seekcount: usize = 0;

    loop {
        let f = File::open(filePath).expect("打开读文件失败！");
        let mut reader = BufReader::new(f);
        reader.seek(SeekFrom::Current(seekcount.try_into().unwrap()));

        loop {
            let mut line = String::new();
            let len = match reader.read_line(&mut line) {
                Ok(len) => len,
                Err(_) => 0,
            };

            if len == 0 {
                break;
            }
            if line.starts_with(regex) {
                tx.send(line);
            }
            //tx.send(line);
            seekcount += len;
        }
        thread::sleep_ms(100);
    }
}

fn write(receive: mpsc::Receiver<String>, filePath: &str) {
    loop {
        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filePath)
            .expect("打开写文件失败！");
        loop {
            let message = receive.recv().unwrap();
            f.write_all(message.as_bytes()).expect("写入文件失败");
            f.flush();
        }
    }
}

#[derive(Deserialize)]
struct Config {
    file: CfgFile,
    filter: CfgFilter,
}

#[derive(Deserialize)]
struct CfgFile {
    input: String,
    output: String,
}

#[derive(Deserialize)]
struct CfgFilter {
    regex: String,
}
