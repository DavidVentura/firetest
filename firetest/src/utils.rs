//use flate2::read::GzDecoder;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use zstd::Decoder;

pub(crate) fn buf_to_fd(name: &str, buf: &[u8]) -> Result<File, Box<dyn Error>> {
    let opts = memfd::MemfdOptions::default().allow_sealing(true);
    let mfd = opts.create(name)?;
    mfd.as_file().set_len(buf.len() as u64)?;
    mfd.add_seals(&[memfd::FileSeal::SealShrink, memfd::FileSeal::SealGrow])?;

    // Prevent further sealing changes.
    mfd.add_seal(memfd::FileSeal::SealSeal)?;
    let mut f = mfd.into_file();

    _ = f.write(&buf)?;
    f.seek(std::io::SeekFrom::Start(0))?;
    Ok(f)
}

pub(crate) fn zstd_buf_to_fd(name: &str, buf: &[u8]) -> Result<File, Box<dyn Error>> {
    let mut zstd_dec = Decoder::new(buf)?;
    let mut dec_buf = Vec::new();
    zstd_dec.read_to_end(&mut dec_buf)?;
    buf_to_fd(name, &dec_buf)
}

pub(crate) fn zstd_dec(buf: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut zstd_dec = Decoder::new(buf)?;
    let mut dec_buf = Vec::new();
    zstd_dec.read_to_end(&mut dec_buf)?;
    Ok(dec_buf)
}
