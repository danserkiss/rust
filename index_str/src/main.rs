use clap::Parser;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::sync::mpsc::{self};
use std::thread;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    //Path to file
    #[arg(short, long)]
    path: String,
}

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
            let _ = write!(f, "{word}:{{\n");
            for (file, indx) in map.iter() {
                let _ = write!(f, "{file}: {indx:?}\n");
                //println!("map:{map:?}\n");
            }
            let _ = write!(f, "}}\n");
        }
        Ok(())
    }
}

fn main() {
    let args = Args::parse();
    parse(&args.path);
}

fn parse(path: &String) {
    let files = fs::read_dir(path);
    if files.is_ok() {
        //dir
        let mut threads = vec![];
        let mut mymap = MyMap::new();
        let (tx, rx) = mpsc::channel();

        for i in files.unwrap() {
            if let Ok(file) = i {
                let filename_opt = file.file_name().clone();
                let filename = filename_opt.into_string().unwrap();
                let mut fullpath = path.clone();
                fullpath.push('/');
                fullpath.push_str(&filename);

                let thread_tx = tx.clone();
                threads.push(thread::spawn(move || {
                    let mut mymap_th = MyMap::new();
                    mymap_th = read_file(&fullpath, &filename, mymap_th);
                    let _ = thread_tx.send(mymap_th.map);
                }));
            }
        }
        drop(tx);
        for recived_map in rx {
            mymap.merge(recived_map);
        }
        for th in threads {
            let _ = th.join();
        }

        println!("{}", mymap);
    } else {
        //file
        let mut mymap = MyMap::new();
        mymap = read_file(&path, &path, mymap);
        println!("{}", mymap);
    }
}

fn read_file(path: &str, filename: &str, mut mymap: MyMap) -> MyMap {
    let contents = fs::read_to_string(&path); // read from file to str
    if contents.is_ok() {
        let text = contents.unwrap();
        mymap = index_words(&text, filename, mymap);
    }
    return mymap;
}

fn index_words(text: &str, filename: &str, mut mymap: MyMap) -> MyMap {
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
