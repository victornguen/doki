use anyhow::Error;
use flate2::read::GzDecoder;
use flate2::Compression;
use log::{log, Level};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;
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
                _ => Err(Error::msg("")),
            }
        })
        .filter_map(|e: Result<(PathBuf, u64), Error>| e.ok())
        .for_each(|(x, size)| {
            if x.is_file() {
                log!(Level::Info, "> Extracted {} ({}) into {}", x.display(), size, target.display())
            }
        });

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

/// Unpacks a zip archive to a target directory
pub fn unpack_zip(zip: &Path, target: &Path) -> Result<(), Error> {
    let file = File::open(zip)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut unpacked = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let save_path = target.join(file.mangled_name());
        let save_path: Rc<Path> = Rc::from(save_path.as_path());
        if file.is_dir() {
            std::fs::create_dir_all(save_path.clone())?;
        } else {
            if let Some(parent) = save_path.clone().parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let mut out = File::create(save_path.clone())?;
            std::io::copy(&mut file, &mut out)?;
            unpacked += 1;
            log!(Level::Info, "> Extracted {} into {}", file.name(), save_path.display());
        }
    }

    log!(Level::Info, "Extracted {} files", unpacked);
    Ok(())
}
