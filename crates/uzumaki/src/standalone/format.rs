use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Magic bytes used at the start and end of an embedded standalone payload.
pub const MAGIC_BYTES: &[u8; 8] = b"uzumaki!";

/// Current standalone format version.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StandaloneMetadata {
    pub format_version: u32,
    pub app_name: String,
    /// Entry point relative to the extracted app root (e.g. `app/index.js`).
    pub entry_rel_path: String,
    /// Name of the root directory the files are extracted into (e.g. `app`).
    pub dist_root_dir_name: String,
    /// Hash of the packed payload, used to derive the extraction directory.
    pub extract_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VfsEntry {
    pub relative_path: String,
    pub offset: u64,
    pub len: u64,
    pub executable: bool,
}

/// Parsed payload read from the tail of an executable.
#[derive(Debug, Clone)]
pub struct StandalonePayload {
    pub metadata: StandaloneMetadata,
    pub manifest: Vec<VfsEntry>,
    pub blob: Vec<u8>,
}

/// Serialize a payload into the appended trailer form. The returned bytes
/// should be appended directly to the base runtime binary.
///
/// Format appended to the host exe:
///   [MAGIC]
///   [metadata_len u64 LE][metadata_json]
///   [manifest_len u64 LE][manifest_json]
///   [blob_len u64 LE][blob]
///   [payload_start u64 LE]    (absolute file offset of the first MAGIC)
///   [MAGIC]
pub fn serialize_payload(
    base_exe_len: u64,
    metadata: &StandaloneMetadata,
    manifest: &[VfsEntry],
    blob: &[u8],
) -> Result<Vec<u8>> {
    let metadata_json = serde_json::to_vec(metadata)?;
    let manifest_json = serde_json::to_vec(manifest)?;

    let mut out: Vec<u8> = Vec::with_capacity(
        MAGIC_BYTES.len()
            + 8 + metadata_json.len()
            + 8 + manifest_json.len()
            + 8 + blob.len()
            + 8 + MAGIC_BYTES.len(),
    );

    // The payload_start written in the trailer is the absolute file offset at
    // which we begin writing the first MAGIC — i.e. base_exe_len.
    out.extend_from_slice(MAGIC_BYTES);
    out.extend_from_slice(&(metadata_json.len() as u64).to_le_bytes());
    out.extend_from_slice(&metadata_json);
    out.extend_from_slice(&(manifest_json.len() as u64).to_le_bytes());
    out.extend_from_slice(&manifest_json);
    out.extend_from_slice(&(blob.len() as u64).to_le_bytes());
    out.extend_from_slice(blob);
    out.extend_from_slice(&base_exe_len.to_le_bytes());
    out.extend_from_slice(MAGIC_BYTES);
    Ok(out)
}

/// Attempt to read a standalone payload from the given executable path.
/// Returns `Ok(None)` if the file does not contain an embedded payload.
pub fn read_payload_from_exe(path: &Path) -> Result<Option<StandalonePayload>> {
    let mut f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let len = f.metadata()?.len();
    let trailer_size: u64 = 8 + MAGIC_BYTES.len() as u64; // payload_start + MAGIC
    if len < trailer_size + MAGIC_BYTES.len() as u64 {
        return Ok(None);
    }

    // Read trailing [payload_start u64][MAGIC]
    f.seek(SeekFrom::End(-(trailer_size as i64)))?;
    let mut trailer = [0u8; 16];
    f.read_exact(&mut trailer)?;
    let payload_start = u64::from_le_bytes(trailer[0..8].try_into().unwrap());
    if &trailer[8..16] != MAGIC_BYTES {
        return Ok(None);
    }
    if payload_start >= len {
        return Ok(None);
    }

    // Seek to payload_start and validate leading MAGIC
    f.seek(SeekFrom::Start(payload_start))?;
    let mut head_magic = [0u8; 8];
    f.read_exact(&mut head_magic)?;
    if &head_magic != MAGIC_BYTES {
        return Ok(None);
    }

    let metadata_json = read_len_prefixed(&mut f)?;
    let manifest_json = read_len_prefixed(&mut f)?;
    let blob = read_len_prefixed(&mut f)?;

    let metadata: StandaloneMetadata = serde_json::from_slice(&metadata_json)
        .context("parsing standalone metadata")?;
    if metadata.format_version != FORMAT_VERSION {
        return Err(anyhow!(
            "unsupported standalone format version: {} (expected {})",
            metadata.format_version,
            FORMAT_VERSION
        ));
    }
    let manifest: Vec<VfsEntry> = serde_json::from_slice(&manifest_json)
        .context("parsing standalone manifest")?;

    Ok(Some(StandalonePayload {
        metadata,
        manifest,
        blob,
    }))
}

fn read_len_prefixed<R: Read>(r: &mut R) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 8];
    r.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

/// Write `bytes` to the file at `path` (appending). Used by the packer.
pub fn append_to_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .with_context(|| format!("opening {} for append", path.display()))?;
    f.write_all(bytes)?;
    Ok(())
}

/// Fast deterministic 64-bit hash (FNV-1a). Avoids extra dependencies. Used
/// only to derive the versioned extraction directory — not security critical.
pub fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}
