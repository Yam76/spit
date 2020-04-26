use structopt::StructOpt;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

const SPITCONFIG: &'static str = ".spitconfig";
const CREATE: &'static str = "create";
const OPEN: &'static str = "open";

#[derive(StructOpt)]
/// Abbreviate frequently typed phrases.
struct Opt {
    /// Create a new config file.
    #[structopt(short, long, conflicts_with("copy"))]
    init: bool,

    /// Copy another config file in FOLDER.
    #[structopt(short, long, name = "FOLDER", conflicts_with("init"))]
    copy: Option<PathBuf>,

    /// All actions use the global instead of the local config file.
    #[structopt(short, long)]
    global: bool,

    /// Assign TEXT to all the supplied names.
    #[structopt(short, long, name="TEXT")]
    add: Option<String>,

    /// Append SEP to the end of each entry.
    #[structopt(default_value = "", short, long, name="SEP")]
    sep: String,

    /// List the available names and their text.
    #[structopt(short, long)]
    list: bool,

    /// List of names.
    #[structopt(name = "NAME")]
    names: Vec<String>,

    /// Warn about invalid names instead of exiting. 
    #[structopt(short, long)]
    warn: bool,

    /// Pass invalid names through instead of exiting. 
    #[structopt(short, long)]
    pass: bool,
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

    // if global then config = global config, else local config
    let config: PathBuf = {
        let mut temp = 
        if opts.global { 
            match dirs::home_dir() {
                Some(v) => v,
                None => {
                    eprintln!("Couldn't find home directory.");
                    std::process::exit(1)
                }
            }
        }
        else { PathBuf::new() };
        temp.push(SPITCONFIG);
        temp
    };

    if let Some(mut path) = opts.copy.clone() {
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
        let mut spit: HashMap<String, String> = deserialize_or_kill(file);
        if opts.list {
            let mut vec: Vec<_> = spit.drain().collect();
            vec.sort_unstable_by(|(name1, _), (name2, _)| name1.cmp(&name2));
            for (name, text) in vec {
                println!("{}: {}", name, text);
            }
        }
        else {
            let spit = spit;
            // if global, then global = empty hashmap else, global = global config 
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

            let mut output = String::new();
            for name in opts.names {
                match spit.get(name.as_str()).or(global.get(name.as_str())) {
                    Some(text) => {
                        output.push_str(&text);
                        output.push_str(&opts.sep);
                    },
                    None => 
                    if opts.pass { 
                        output.push_str(&name);
                        output.push_str(&opts.sep);
                        if opts.warn { // pass and warn = out and warn
                            eprintln!("Couldn't find {}", name)
                        }
                        // pass and no warn = only out
                    }
                    else { 
                        eprintln!("Couldn't find {}", name); // no pass and warn = only warn
                        if !opts.warn { // no pass and no warn = quit
                            std::process::exit(1)
                        }
                    }
                }
            }
            print!("{}", output);
        }

    }
}
