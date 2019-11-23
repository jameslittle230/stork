use std::env;
use std::fs;
use serde::Deserialize;
use std::io::BufReader;
use std::io::Read;
use std::fs::File;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Config {
    scope: String
}

struct Entry {

}

struct Field {

}

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = &args[1];
    let argument = &args[2];
    // let debug = &args[3];

    // Read configuration file
    let contents = fs::read_to_string(argument)
        .expect(&std::format!("Something went wrong reading the file {}", argument));
    let config: Config = toml::from_str(&contents)
        .expect("Config file does not contain proper TOML syntax.");

    let mut output: HashMap<String, String> = HashMap::new();
    
    // Find directory to scan
    let filepaths = fs::read_dir(config.scope).unwrap();
    for path in filepaths {
        // For each file in that directory:
        let pathname = path.unwrap().path();
        println!("{}", pathname.display());

        // For each word
        let file = File::open(&pathname).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        buf_reader.read_to_string(&mut contents);
        let word_vector: Vec<&str> = contents.split_whitespace().collect();
        for word in word_vector {
            output.insert(word.to_string(), pathname.to_str().unwrap().to_string());
        }

        for key in output.keys() {
            println!("{}: {}", key, output.get(key).unwrap_or(&"NOTHING".to_owned()));
        }
    
        // If output[word] does not have file:
        
            // Add (file, 1) to output[word]
    
        // If output[word] already has file:
    
            // Increment output[word][file]

    }

        
    // Serialize output file

    // Write output file to disk
}