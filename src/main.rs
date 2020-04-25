use structopt::StructOpt;
use std::collections::HashMap;
use std::fs::OpenOptions;

const SPITCONFIG: &'static str = ".spitconfig";
const CREATE: &'static str = "CREATE";
const OPEN: &'static str = "OPEN";

#[derive(StructOpt)]
/// Abbreviate frequently typed phrases.
struct Opt {
    /// Create a new config file.
    #[structopt(long)]
    init: bool,

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

fn kill_or(verb: &str, oo: &mut std::fs::OpenOptions) -> std::fs::File {
    match oo.open(SPITCONFIG) {
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
    if opts.init { // initialize new .spitconfig
        let file = kill_or(CREATE, OpenOptions::new().write(true).create_new(true));
        serialize_or_kill(file, &HashMap::<String, String>::new());
    }
    else if let Some(text) = opts.add { // add to .spitconfig
        let read_file = kill_or(OPEN, OpenOptions::new().read(true));
        let mut spit: HashMap<String, String> = deserialize_or_kill(read_file);
        for name in opts.names {
            spit.insert(name, text.clone()); // maybe add verbose here?
        }
        let write_file = kill_or(OPEN, OpenOptions::new().write(true));
        serialize_or_kill(write_file, &spit);
    }
    else {
        let file = kill_or(OPEN, OpenOptions::new().read(true));
        let spit: HashMap<String, String> = deserialize_or_kill(file);
        for name in opts.names {
            match spit.get(name.as_str()) {
                None => eprintln!("Couldn't find {}", name),
                Some(v) => print!("{}{}", v, opts.sep)
            }
        }
    }
}
