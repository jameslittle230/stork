use super::structs::Index;
use crate::config::Config;
use std::fs::File;
use std::io::{BufWriter, Write};

fn write_release(index: &Index, bufwriter: &mut BufWriter<File>, write_version: &[u8]) -> usize {
    let mut bytes_written: usize = 0;

    let entries_encoded = bincode::serialize(&index.entries).unwrap();
    let results_encoded = bincode::serialize(&index.queries).unwrap();
    let byte_vectors_to_write = [
        write_version,
        entries_encoded.as_slice(),
        results_encoded.as_slice(),
    ];

    for vec in byte_vectors_to_write.iter() {
        bytes_written += bufwriter.write(&(vec.len() as u64).to_be_bytes()).unwrap();
        bytes_written += bufwriter.write(vec).unwrap();
    }

    bytes_written
}

fn write_debug(index: &Index, bufwriter: &mut BufWriter<File>, write_version: &[u8]) {
    let entries_encoded = serde_json::to_string_pretty(&index.entries).unwrap();
    let results_encoded = serde_json::to_string_pretty(&index.queries).unwrap();
    let byte_vectors_to_write = [
        write_version,
        entries_encoded.as_bytes(),
        results_encoded.as_bytes(),
    ];

    for vec in byte_vectors_to_write.iter() {
        let _ = bufwriter.write(vec.len().to_string().as_bytes());
        let _ = bufwriter.write(b"\n");
        let _ = bufwriter.write(vec);
        let _ = bufwriter.write(b"\n\n");
    }
}

impl Index {
    pub fn write(&self, config: &Config) -> usize {
        let config = &config.output;
        let file = File::create(&config.filename).unwrap();
        let mut bufwriter = BufWriter::new(file);
        let write_version = super::VERSION_STRING.as_bytes();

        if config.debug.unwrap_or(false) {
            write_debug(self, &mut bufwriter, &write_version); 0
        } else {
            write_release(self, &mut bufwriter, &write_version)
        }
    }
}
