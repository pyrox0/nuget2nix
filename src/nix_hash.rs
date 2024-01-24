use camino::Utf8PathBuf;
use sha2::{Digest, Sha256};
use nix_base32::to_nix_base32;
use std::{
    fs::File,
    io::{BufReader, Read},
};

pub fn hash(path: &Utf8PathBuf) -> anyhow::Result<String> {
    let input = File::open(path)?;
    let mut reader = BufReader::new(input);

    let mut context = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(to_nix_base32(context.finalize().as_ref()))
}
