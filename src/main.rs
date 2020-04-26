use structopt::StructOpt;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
#[macro_use]
extern crate lazy_static;

type SpitMap = HashMap<String, String>;

const SPITCONFIG: &'static str = ".spitconfig";
const CREATE: &'static str = "create";
const OPEN: &'static str = "open";

lazy_static!{
    static ref READ_OPTIONS: OpenOptions = {
        let mut temp = OpenOptions::new();
        temp.read(true);
        temp
    };
    static ref CREATE_OPTIONS: OpenOptions = {
        let mut temp = OpenOptions::new();
        temp.write(true).create_new(true);
        temp
    };
}

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

    #[structopt(flatten)]
    process: ProcessOpt,
}

#[derive(StructOpt, Clone, Copy)]
struct ProcessOpt {
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

fn deserialize_or_kill(file: std::fs::File) -> SpitMap {
    match serde_json::from_reader(file) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Could not deserialize '{}': {}", SPITCONFIG, e);
            std::process::exit(1)
        }
    }
}

fn serialize_or_kill(file: std::fs::File, map: &SpitMap) {
    if let Err(e) = serde_json::to_writer(file, &map) {
        eprintln!("Could not write to '{}': {}", SPITCONFIG, e);
        std::process::exit(1)
    }
}

fn spit_out(spit: SpitMap, global: SpitMap, names: Vec<String>, sep: String, opts: ProcessOpt) {
    let mut output = String::new();
    for name in names {
        match spit.get(name.as_str()).or(global.get(name.as_str())) {
            Some(text) => {
                output.push_str(&text);
                output.push_str(&sep);
            },
            None => 
            if opts.pass { 
                output.push_str(&name);
                output.push_str(&sep);
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

fn get_config(global: bool) -> PathBuf {
    let mut temp = 
    if global { 
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
}

fn get_global(global: bool) -> SpitMap {
    // if global, then global = empty hashmap else, global = global config     
    if global { HashMap::new() }
    else {
        dirs::home_dir()
        .map_or(HashMap::new(),
        |mut path| {
            path.push(SPITCONFIG);    
            READ_OPTIONS.open(path)
            .map_or(HashMap::new(), 
            |file| { 
                serde_json::from_reader(file).unwrap_or(HashMap::new()) 
            })
        })
    }   
}

fn add_to_config(text: String, names: Vec<String>, config: PathBuf) {
    let read_file = kill_or(OPEN, &READ_OPTIONS, &config);
    let mut spit: SpitMap = deserialize_or_kill(read_file);
    // if pass allow replacement
    // if warning warn when replace
    for name in names {
        spit.insert(name, text.clone()); // maybe add verbose here?
    }
    let write_file = kill_or(OPEN, &OpenOptions::new().write(true), &config);
    serialize_or_kill(write_file, &spit);
}

fn print_alphabetical(mut spit: SpitMap) {
    let mut vec: Vec<_> = spit.drain().collect();
    vec.sort_unstable_by(|(name1, _), (name2, _)| name1.cmp(&name2));
    for (name, text) in vec {
        println!("{}: {}", name, text);
    }
}

fn copy_config(mut path: PathBuf, config: &PathBuf) {
    path.push(SPITCONFIG);
    let mut origin = kill_or("find", &READ_OPTIONS, &path);
    let mut target = kill_or(CREATE, &CREATE_OPTIONS, config);
    if let Err(e) = std::io::copy(&mut origin, &mut target) {
        eprintln!("Could not write to '{}': {}", SPITCONFIG, e);
        std::process::exit(1)
    };
}

fn init_config(config: &PathBuf) {
    serialize_or_kill(kill_or(CREATE, &CREATE_OPTIONS, config), &SpitMap::new())
}

fn main() {
    let opts = Opt::from_args();
    // if global then config = global config, else local config
    let config: PathBuf = get_config(opts.global);

    // copy existing .spitconfig
    if let Some(path) = opts.copy { copy_config(path, &config) }
    // initialize new .spitconfig
    else if opts.init { init_config(&config) };

    // add to .spitconfig
    if let Some(text) = opts.add { add_to_config(text, opts.names, config) }
    else { 
        let spit = deserialize_or_kill(kill_or(OPEN, &READ_OPTIONS, &config));
        // list names and corresponding text
        if opts.list { print_alphabetical(spit) }
        // output corresponding text of given names
        else { spit_out(spit, get_global(opts.global), opts.names, opts.sep, opts.process) }
    }
}
