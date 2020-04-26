use structopt::StructOpt;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

const SPITCONFIG: &'static str = ".spitconfig";
const CREATE: &'static str = "CREATE";
const OPEN: &'static str = "OPEN";

#[derive(StructOpt)]
/// Abbreviate frequently typed phrases.
struct Opt {
    /// Create a new config file.
    #[structopt(long)]
    init: bool,

    /// Copy another config file in FOLDER.
    #[structopt(long, name = "FOLDER")]
    copy: Option<PathBuf>,

    /// All actions apply to the global config file.
    #[structopt(short, long)]
    global: bool,

    /// Assign TEXT to all the supplied names.
    #[structopt(short, long, name="TEXT")]
    add: Option<String>,

    /// Append SEP to the end of each entry.
    #[structopt(default_value = "", short, long, name="SEP")]
    sep: String,

    /// List of names.
    #[structopt(name = "NAME")]
    names: Vec<String>,
}

fn kill_or(verb: &str, oo: &std::fs::OpenOptions, config: &Path) -> std::fs::File {
    match oo.open(config) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Could not {} '{}': {}", verb, SPITCONFIG, e);
            std::process::exit(1)
        }
    }
}

fn deserialize_or_kill(file: std::fs::File) -> HashMap<String, String> {
    match serde_json::from_reader(file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Could not deserialize '{}': {}", SPITCONFIG, e);
            std::process::exit(1)
        }
    }
}

fn serialize_or_kill(file: std::fs::File, map: &HashMap<String, String>) {
    if let Err(e) = serde_json::to_writer(file, &map) {
        eprintln!("Could not write to '{}': {}", SPITCONFIG, e);
        std::process::exit(1)
    }
}

fn main() {
    let opts = Opt::from_args();
    let read_options = {
        let mut temp = OpenOptions::new();
        temp.read(true);
        temp
    };
    let create_options = {
        let mut temp = OpenOptions::new();
        temp.write(true).create_new(true);
        temp
    };

    let config: PathBuf = 
    {
        let mut temp = 
        if opts.global { // be lazy and don't alert them that home doesn't exist TODO
            dirs::home_dir().unwrap_or(PathBuf::new())
        }
        else { PathBuf::new() };
        temp.push(SPITCONFIG);
        temp
    };
    /* if global, then 
        config = global config
        global = empty hashmap
        else,
        config = local config
        global = global config
    */




    if let Some(mut path) = opts.copy {
        path.push(SPITCONFIG);
        let mut origin = kill_or("find", &read_options, &path);
        let mut target = kill_or(CREATE, &create_options, &config);
        if let Err(e) = std::io::copy(&mut origin, &mut target) {
            eprintln!("Could not write to '{}': {}", SPITCONFIG, e);
            std::process::exit(1)
        };
    }
    else if opts.init { // initialize new .spitconfig
        let file = kill_or(CREATE, &create_options, &config);
        serialize_or_kill(file, &HashMap::<String, String>::new());
    }

    if let Some(text) = opts.add { // add to .spitconfig
        let read_file = kill_or(OPEN, &read_options, &config);
        let mut spit: HashMap<String, String> = deserialize_or_kill(read_file);
        for name in opts.names {
            spit.insert(name, text.clone()); // maybe add verbose here?
        }
        let write_file = kill_or(OPEN, &OpenOptions::new().write(true), &config);
        serialize_or_kill(write_file, &spit);
    }
    else { // no options
        let file = kill_or(OPEN, &read_options, &config);
        let spit: HashMap<String, String> = deserialize_or_kill(file);
        let global: HashMap<String, String> = 
        if opts.global { HashMap::new() }
        else {
            dirs::home_dir()
            .map_or(HashMap::new(),
            |mut path| {
                path.push(SPITCONFIG);    
                read_options.open(path)
                .map_or(HashMap::new(),
                |file| { 
                    serde_json::from_reader(file).unwrap_or(HashMap::new()) 
                })
            })
        };
        for name in opts.names {
            match spit.get(name.as_str()).or(global.get(name.as_str())) {
                None => eprintln!("Couldn't find {}", name),
                Some(v) => print!("{}{}", v, opts.sep)
            }
        }
    }
}
