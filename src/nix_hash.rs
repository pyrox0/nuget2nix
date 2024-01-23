use nix_base32::to_nix_base32;
use ring::digest::{Context, Digest, SHA256};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

pub fn hash(path: &PathBuf) -> anyhow::Result<String> {
    let input = File::open(path)?;
    let reader = BufReader::new(input);
    let digest = sha256_digest(reader);

    Ok(to_nix_base32(digest?.as_ref()))
}

fn sha256_digest<R: Read>(mut reader: R) -> anyhow::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}
