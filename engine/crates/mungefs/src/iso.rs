//! Minimal ISO9660 reader — enough to enumerate and read files from the PS2
//! game discs so `bake` can operate directly on an .iso (users supply their
//! own disc image; nothing is written back).
//!
//! Supports: primary volume descriptor, nested directory records, ";1"
//! version suffixes. No Joliet/Rock Ridge (the PS2 discs don't need them).

use crate::UcfbError;
use std::io::{Read, Seek, SeekFrom};

const SECTOR: u64 = 2048;

/// Sanity caps on the directory tree. A disc image is untrusted input: an
/// extent size is a raw u32 (4 GiB if believed), and a directory whose extent
/// LBA points back at itself or an ancestor would walk forever, growing the
/// path string and the pending stack on every pass. The shipped discs hold
/// ~220 files in a dozen directories, so both caps are far above anything real.
const MAX_DIR_EXTENT: u32 = 4 << 20;
const MAX_DIRS: usize = 4096;

/// Fixed part of an ISO9660 directory record, before the name bytes.
const REC_HEADER: usize = 33;

pub struct IsoFile {
    /// Path with '/' separators, version suffix stripped, e.g.
    /// "DATA/_LVL_PS2/MISSION.LVL".
    pub path: String,
    pub lba: u32,
    pub size: u32,
}

pub struct Iso<R> {
    reader: R,
    pub files: Vec<IsoFile>,
}

impl<R: Read + Seek> Iso<R> {
    pub fn open(mut reader: R) -> Result<Iso<R>, UcfbError> {
        // PVD at sector 16
        let mut pvd = [0u8; 2048];
        read_at(&mut reader, 16 * SECTOR, &mut pvd)?;
        if &pvd[1..6] != b"CD001" || pvd[0] != 1 {
            return Err(UcfbError::BadMagic);
        }
        // Root directory record at offset 156
        let root_lba = u32::from_le_bytes(pvd[158..162].try_into().unwrap());
        let root_size = u32::from_le_bytes(pvd[166..170].try_into().unwrap());

        let mut files = Vec::new();
        let mut stack = vec![(String::new(), root_lba, root_size)];
        let mut visited = std::collections::HashSet::new();
        while let Some((prefix, lba, size)) = stack.pop() {
            // An extent LBA that points at a directory already walked (itself,
            // or an ancestor) is a cycle: descending again would never end.
            if !visited.insert(lba) {
                continue;
            }
            if visited.len() > MAX_DIRS {
                return Err(UcfbError::Malformed(format!(
                    "iso: more than {MAX_DIRS} directories"
                )));
            }
            if size > MAX_DIR_EXTENT {
                return Err(UcfbError::Malformed(format!(
                    "iso: directory extent of {size} bytes at LBA {lba}"
                )));
            }
            let mut dir = vec![0u8; size as usize];
            read_at(&mut reader, lba as u64 * SECTOR, &mut dir)?;
            let mut off = 0usize;
            while off < dir.len() {
                let base = lba as usize * SECTOR as usize + off;
                let len = dir[off] as usize;
                if len == 0 {
                    // records don't cross sector boundaries; skip to next
                    off = (off / 2048 + 1) * 2048;
                    continue;
                }
                // The record length is file data: believing it blindly slices
                // past the extent, and any record shorter than the fixed
                // header plus one name byte would index past its own end.
                if len < REC_HEADER + 1 || off + len > dir.len() {
                    return Err(UcfbError::Truncated(base));
                }
                let rec = &dir[off..off + len];
                off += len;
                let name_len = rec[32] as usize;
                if REC_HEADER + name_len > len {
                    return Err(UcfbError::Truncated(base));
                }
                let name_raw = &rec[REC_HEADER..REC_HEADER + name_len];
                if name_raw == [0] || name_raw == [1] {
                    continue; // "." and ".."
                }
                let mut name = String::from_utf8_lossy(name_raw).into_owned();
                if let Some(p) = name.find(';') {
                    name.truncate(p);
                }
                let child_lba = u32::from_le_bytes(rec[2..6].try_into().unwrap());
                let child_size = u32::from_le_bytes(rec[10..14].try_into().unwrap());
                let flags = rec[25];
                let path = if prefix.is_empty() {
                    name
                } else {
                    format!("{prefix}/{name}")
                };
                if flags & 0x02 != 0 {
                    stack.push((path, child_lba, child_size));
                } else {
                    files.push(IsoFile {
                        path,
                        lba: child_lba,
                        size: child_size,
                    });
                }
            }
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Iso { reader, files })
    }

    /// Read a whole file by its listing entry.
    pub fn read_file(&mut self, f: &IsoFile) -> Result<Vec<u8>, UcfbError> {
        let mut buf = vec![0u8; f.size as usize];
        read_at(&mut self.reader, f.lba as u64 * SECTOR, &mut buf)?;
        Ok(buf)
    }

    /// Find a file by exact path (case-insensitive).
    pub fn find(&self, path: &str) -> Option<usize> {
        self.files
            .iter()
            .position(|f| f.path.eq_ignore_ascii_case(path))
    }
}

fn read_at<R: Read + Seek>(r: &mut R, pos: u64, buf: &mut [u8]) -> Result<(), UcfbError> {
    r.seek(SeekFrom::Start(pos))
        .and_then(|_| r.read_exact(buf))
        .map_err(|_| UcfbError::Truncated(pos as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A minimal image: PVD at sector 16 pointing at one directory extent at
    /// sector 17, whose bytes are `dir`.
    fn image(root_size: u32, dir: &[u8]) -> Cursor<Vec<u8>> {
        let mut img = vec![0u8; 19 * 2048];
        let pvd = 16 * 2048;
        img[pvd] = 1;
        img[pvd + 1..pvd + 6].copy_from_slice(b"CD001");
        img[pvd + 158..pvd + 162].copy_from_slice(&17u32.to_le_bytes());
        img[pvd + 166..pvd + 170].copy_from_slice(&root_size.to_le_bytes());
        img[17 * 2048..17 * 2048 + dir.len()].copy_from_slice(dir);
        Cursor::new(img)
    }

    /// `Iso` is not `Debug` (R need not be), so unwrap the error by hand.
    fn open_err(root_size: u32, dir: &[u8]) -> UcfbError {
        match Iso::open(image(root_size, dir)) {
            Err(e) => e,
            Ok(_) => panic!("expected an error"),
        }
    }

    /// A record-length byte longer than the extent used to slice past it.
    #[test]
    fn record_running_past_the_extent_errors() {
        let e = open_err(32, &[200]);
        assert!(matches!(e, UcfbError::Truncated(_)), "{e}");
    }

    /// A record shorter than the fixed 33-byte header used to index past it
    /// when reading the name length, the child LBA and the flags.
    #[test]
    fn record_shorter_than_the_header_errors() {
        let e = open_err(2048, &[10]);
        assert!(matches!(e, UcfbError::Truncated(_)), "{e}");
        // Length 34 but a name length that runs past the record end.
        let mut rec = vec![0u8; 34];
        rec[0] = 34;
        rec[32] = 200;
        let e = open_err(2048, &rec);
        assert!(matches!(e, UcfbError::Truncated(_)), "{e}");
    }

    /// A directory whose extent LBA is its own used to loop forever, growing
    /// the path string and the pending stack every pass.
    #[test]
    fn self_referential_directory_terminates() {
        let mut rec = vec![0u8; 35];
        rec[0] = 35;
        rec[2..6].copy_from_slice(&17u32.to_le_bytes()); // child LBA = this dir
        rec[10..14].copy_from_slice(&2048u32.to_le_bytes());
        rec[25] = 0x02; // directory
        rec[32] = 1;
        rec[33] = b'A';
        let Ok(iso) = Iso::open(image(2048, &rec)) else {
            panic!("walk terminates without error")
        };
        assert!(iso.files.is_empty(), "the only entry is the cycling dir");
    }

    /// The extent size is a raw u32; 0xFFFF_FFFF used to be allocated whole.
    #[test]
    fn absurd_directory_extent_errors() {
        let e = open_err(u32::MAX, &[]);
        assert!(matches!(e, UcfbError::Malformed(_)), "{e}");
    }
}
