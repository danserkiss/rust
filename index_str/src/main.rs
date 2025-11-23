use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::task;
// use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    //Path to file
    #[arg(short, long)]
    path: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct MyMap {
    map: HashMap<String, HashMap<String, Vec<usize>>>,
}
impl MyMap {
    fn new() -> MyMap {
        MyMap {
            map: HashMap::new(),
        }
    }
    fn merge(&mut self, other: HashMap<String, HashMap<String, Vec<usize>>>) {
        for (word, map) in other {
            let entry = self.map.entry(word).or_insert_with(|| HashMap::new());
            for (file, indx) in map {
                entry.entry(file).or_insert_with(|| Vec::new()).extend(indx);
            }
        }
    }
}

impl fmt::Display for MyMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (word, map) in self.map.iter() {
            let _ = write!(f, "\"{word}\":{{\n");
            for (file, indx) in map.iter() {
                let _ = write!(f, "\"{file}\": {indx:?},\n");
            }
            let _ = write!(f, "}}\n");
        }
        Ok(())
    }
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut path_buf: Vec<String> = Vec::new();
    let mut mymap: MyMap = MyMap::new();

    collet_files(&args.path, &mut path_buf).await;
    let (tx, mut rx) = mpsc::unbounded_channel::<MyMap>();
    let handle = task::spawn(async move {
        while let Some(_msg) = rx.recv().await {
            mymap.merge(_msg.map);
        }
        mymap
    });
    for i in path_buf.into_iter() {
        let tx_clone = tx.clone();
        let _ = tokio::spawn(async move {
            let val = index_file(i.clone()).await;
            let _ = tx_clone.send(val);
        });
    }
    drop(tx);

    mymap = handle.await.unwrap();
    let mut file = tokio::fs::File::create("index_result.json").await.unwrap();
    println!("{}\n", mymap);
    let json = serde_json::to_string(&mymap).unwrap();
    file.write(json.as_bytes()).await.unwrap();
}

async fn collet_files(path: &String, path_buf: &mut Vec<String>) {
    let directory = fs::read_dir(path);
    if directory.is_ok() {
        //dir
        for i in directory.unwrap() {
            if let Ok(file) = i {
                let filename_opt = file.file_name().clone();
                let filename = filename_opt.into_string().unwrap();
                let mut fullpath = path.clone();
                fullpath.push('/');
                fullpath.push_str(&filename);
                if let Ok(filetype) = file.file_type() {
                    if filetype.is_dir() {
                        let mut rec_vec = Vec::new();
                        Box::pin(collet_files(&fullpath, &mut rec_vec)).await;
                        path_buf.append(&mut rec_vec);
                    } else if filetype.is_file() {
                        path_buf.push(fullpath);
                    }
                }
            }
        }
    } else {
        //file
        path_buf.push(path.clone());
    }
}

async fn index_file(path: String) -> MyMap {
    // println!("Before sleep");
    // tokio::time::sleep(Duration::from_secs(1)).await;
    // println!("After sleep");
    let mut mymap = MyMap::new();
    let vec: Vec<&str> = path.rsplit('/').collect();

    let contents = tokio::fs::read_to_string(&path).await; // read from file to str
    if contents.is_ok() {
        let text = contents.unwrap();
        mymap = index_words(&text, vec[0], mymap).await;
    }
    return mymap;
}

async fn index_words(text: &str, filename: &str, mut mymap: MyMap) -> MyMap {
    let mut i = 0;
    for word in text.split_whitespace() {
        let word_map = mymap
            .map
            .entry(word.to_string())
            .or_insert_with(|| HashMap::new());
        word_map.entry(filename.to_string()).or_default().push(i);

        i += word.len() + 1;
    }
    return mymap;
}
