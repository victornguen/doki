use anyhow::Error;
use flate2::read::GzDecoder;
use flate2::Compression;
use log::{log, Level};
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::EntryType;

/// Unpacks a tarball to a target directory
pub fn unpack_tar_gz(tarball: &Path, target: &Path) -> Result<(), Error> {
    let tar_gz = File::open(tarball)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    let mut unpacked = 0;
    archive
        .entries()?
        .filter_map(|e| e.ok())
        .map(|mut entry| {
            let entry_path = entry.path()?;
            // let entry_path = entry_path.strip_prefix(entry_path.iter().next().expect("Failed to get entry root prefix"))?;
            match entry.header().entry_type() {
                EntryType::Regular | EntryType::Directory => {
                    let save_path = target.join(entry_path);
                    entry.unpack(save_path.clone())?;
                    unpacked += 1;
                    Ok((save_path, entry.size()))
                }
                _ => { Err(Error::msg("")) }
            }
        })
        .filter_map(|e: Result<(PathBuf, u64), Error>| e.ok())
        .for_each(|(x, size)|
            if x.is_file() { log!(Level::Info, "> Extracted {} ({}) into {}", x.display(), size, target.display()) }
        );

    log!(Level::Info, "Extracted {} files", unpacked);
    Ok(())
}


/// Packs a target directory into a tarball
pub fn pack_tar_gz(target: &Path, tarball: &Path, level: Compression) -> Result<(), Error> {
    let tar_gz = File::create(tarball)?;
    let enc = flate2::write::GzEncoder::new(tar_gz, level);
    let mut tar = tar::Builder::new(enc);

    tar.append_dir_all("", target)?;
    tar.finish()?;

    Ok(())
}


/// Unpacks a rar archive to a target directory
pub fn unpack_rar(rar: &Path, target: &Path) -> Result<(), Error> {
    let mut archive = unrar::Archive::new(rar).open_for_processing().expect("Open archive for processing");
    let mut unpacked = 0;
    while let Some(header) = archive.read_header()? {
        let entry = header.entry();

        archive = if entry.is_file() {
            log!(Level::Info, "Extracting {} ({}) into {}", entry.filename.display(), entry.unpacked_size, target.display());
            unpacked += 1;
            header.extract_with_base(target)?
        } else {
            header.skip()?
        };
    }
    log!(Level::Info, "Extracted {} files", unpacked);
    Ok(())
}