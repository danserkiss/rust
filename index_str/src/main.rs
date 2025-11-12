use clap::Parser;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    //Path to file
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    max_th: usize,
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
            let _ = write!(f, "\"{word}\":{{\n");
            for (file, indx) in map.iter() {
                let _ = write!(f, "\"{file}\": {indx:?},\n");
            }
            let _ = write!(f, "}}\n");
        }
        Ok(())
    }
}

fn main() {
    let args = Args::parse();
    let mut path_buf: Vec<String> = Vec::new();
    collet_files(&args.path, &mut path_buf);
    let tasks = Arc::new(Mutex::new(path_buf.into_iter()));
    let result = Arc::new(Mutex::new(Vec::new()));

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..args.max_th {
            let tasks_cl = Arc::clone(&tasks);
            let result_cl = Arc::clone(&result);
            let handle = scope.spawn(move || {
                let mut th_res = Vec::new();
                loop {
                    let next_task = {
                        let mut task_guard = tasks_cl.lock().unwrap();
                        task_guard.next()
                    };
                    match next_task {
                        Some(task_data) => {
                            let result = index_file(task_data.as_str());
                            th_res.push(result);
                        }
                        None => {
                            break;
                        }
                    }
                }
                let mut result_guard = result_cl.lock().unwrap();
                result_guard.extend(th_res);
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
    });

    let mut mymap: MyMap = MyMap::new();
    for i in result.lock().unwrap().iter() {
        mymap.merge(i.map.clone());
    }
    let mut file = File::create("index_result.json").unwrap();
    println!("{}", mymap);
    let _ = writeln!(file, "{}", mymap);
}
fn collet_files(path: &String, path_buf: &mut Vec<String>) {
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
                        collet_files(&fullpath, &mut rec_vec);
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

fn index_file(path: &str) -> MyMap {
    let mut mymap = MyMap::new();
    let vec: Vec<&str> = path.rsplit('/').collect();

    let contents = fs::read_to_string(&path); // read from file to str
    if contents.is_ok() {
        let text = contents.unwrap();
        mymap = index_words(&text, vec[0], mymap);
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
