use std::{
    fmt,
    io::{Read, Write},
    str::FromStr,
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use thiserror::Error;

use crate::repo::Repo;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ObjectError {
    #[error("invalid object id: {0}")]
    InvalidOid(String),
    #[error("object not found: {0}")]
    NotFound(ObjectId),
    #[error("corrupt object: {0}")]
    Corrupt(String),
    #[error("unknown object kind: {0}")]
    UnknownKind(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ── ObjectId ──────────────────────────────────────────────────────────────────

/// A 20-byte SHA1 hash that uniquely identifies a git object.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId([u8; 20]);

impl ObjectId {
    /// Construct an ObjectId from a 20-byte array.
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        ObjectId(bytes)
    }

    /// The first byte as hex (2 chars) — used as the fan-out directory name.
    pub fn fan_out(&self) -> String {
        hex::encode(&self.0[..1])
    }

    /// The remaining 19 bytes as hex (38 chars) — used as the filename.
    pub fn remainder(&self) -> String {
        hex::encode(&self.0[1..])
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectId({self})")
    }
}

impl FromStr for ObjectId {
    type Err = ObjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ObjectError::InvalidOid(s.to_string()))?;
        let arr: [u8; 20] = bytes
            .try_into()
            .map_err(|_| ObjectError::InvalidOid(s.to_string()))?;
        Ok(ObjectId(arr))
    }
}

// ── ObjectKind ────────────────────────────────────────────────────────────────

/// The four git object types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl ObjectKind {
    /// The ASCII string git uses in the object header, e.g. `"blob"`.
    pub fn as_str(self) -> &'static str {
        match self {
            ObjectKind::Blob => "blob",
            ObjectKind::Tree => "tree",
            ObjectKind::Commit => "commit",
            ObjectKind::Tag => "tag",
        }
    }
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ObjectKind {
    type Err = ObjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectKind::Blob),
            "tree" => Ok(ObjectKind::Tree),
            "commit" => Ok(ObjectKind::Commit),
            "tag" => Ok(ObjectKind::Tag),
            other => Err(ObjectError::UnknownKind(other.to_string())),
        }
    }
}

// ── Object ────────────────────────────────────────────────────────────────────

/// A git object: a kind and its raw content bytes.
pub struct Object {
    pub kind: ObjectKind,
    pub data: Vec<u8>,
}

impl Object {
    pub fn new(kind: ObjectKind, data: Vec<u8>) -> Self {
        Object { kind, data }
    }

    /// Serialize to the git object format: `"<kind> <len>\0<data>"`.
    pub fn to_store_bytes(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.kind, self.data.len());
        let mut buf = Vec::with_capacity(header.len() + self.data.len());
        buf.extend_from_slice(header.as_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    /// Compute the SHA1 of this object's store representation.
    pub fn id(&self) -> ObjectId {
        let store_bytes = self.to_store_bytes();
        let hash: [u8; 20] = Sha1::digest(&store_bytes).into();
        ObjectId::from_bytes(hash)
    }
}

// ── Store ─────────────────────────────────────────────────────────────────────

/// The loose-object store backed by `.git/objects/`.
pub struct Store<'a> {
    repo: &'a Repo,
}

impl<'a> Store<'a> {
    pub fn new(repo: &'a Repo) -> Self {
        Store { repo }
    }

    fn objects_dir(&self) -> std::path::PathBuf {
        self.repo.git_dir().join("objects")
    }

    /// Path to a loose object file given its id.
    fn object_path(&self, id: &ObjectId) -> std::path::PathBuf {
        self.objects_dir()
            .join(id.fan_out())
            .join(id.remainder())
    }

    /// Write an object to the store. Returns its `ObjectId`.
    pub fn write(&self, object: &Object) -> Result<ObjectId, ObjectError> {
        let store_bytes = object.to_store_bytes();
        let obj_id = object.id();
        let obj_path = self.object_path(&obj_id);

        if obj_path.exists() {
            return Ok(obj_id)
        }

        let fanout_dir_path = obj_path.parent().unwrap();
        std::fs::create_dir_all(fanout_dir_path).map_err(ObjectError::Io)?;

        let buff = Vec::new();
        let mut z = ZlibEncoder::new(buff, Compression::default());
        z.write_all(&store_bytes).map_err(ObjectError::Io)?;
        let compressed_bytes = z.finish().map_err(ObjectError::Io)?;

        std::fs::write(obj_path, &compressed_bytes)
            .map_err(ObjectError::Io)?;

        Ok(obj_id)
    }

    /// Read and parse an object from the store by its id.
    pub fn read(&self, id: &ObjectId) -> Result<Object, ObjectError> {
        let obj_path = self.object_path(id);

        let obj_bytes = std::fs::read(&obj_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ObjectError::NotFound(*id)
            } else {
                ObjectError::Io(e)
            }
        })?;

        let mut z = ZlibDecoder::new(&obj_bytes[..]);
        let mut decoded_bytes = Vec::new();
        z.read_to_end(&mut decoded_bytes)?; 

        let null_index = decoded_bytes.iter().position(|&x| x == b'\0')
            .ok_or(ObjectError::Corrupt("obj bytes missing null separator".to_string()))?;

        let header = decoded_bytes[..null_index].to_vec();
        let space_idx = header.iter().position(|&x| x == b' ')
            .ok_or(ObjectError::Corrupt("header missing space separator".to_string()))?;

        let kind_bytes = &header[..space_idx];
        let len_str = std::str::from_utf8(&header[space_idx + 1..])
            .map_err(|_| ObjectError::Corrupt("header length is not valid UTF-8".to_string()))?;
        let obj_len: usize = len_str
            .parse()
            .map_err(|_| ObjectError::Corrupt(format!("invalid length in header: {len_str}")))?;

        let data = decoded_bytes[null_index + 1..].to_vec();

        if obj_len != data.len() {
            return Err(ObjectError::Corrupt(format!(
                "header says {obj_len} bytes but got {}",
                data.len()
            )));
        }

        let kind = ObjectKind::from_str(
            std::str::from_utf8(kind_bytes)
                .map_err(|_| ObjectError::Corrupt("kind is not valid UTF-8".to_string()))?,
        )?;

        Ok(Object { kind, data })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands::init, repo::Repo};
    use tempfile::TempDir;

    fn make_repo() -> (TempDir, Repo) {
        let dir = TempDir::new().unwrap();
        init::run(dir.path()).unwrap();
        let repo = Repo::discover(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn object_id_roundtrip() {
        let hex = "ce013625030ba8dba906f756967f9e9ca394464a";
        let oid: ObjectId = hex.parse().unwrap();
        assert_eq!(oid.to_string(), hex);
    }

    #[test]
    fn object_id_fan_out() {
        let oid: ObjectId = "ce013625030ba8dba906f756967f9e9ca394464a".parse().unwrap();
        assert_eq!(oid.fan_out(), "ce");
        assert_eq!(oid.remainder(), "013625030ba8dba906f756967f9e9ca394464a");
    }

    #[test]
    fn blob_store_bytes() {
        // "blob 5\0hello" is the git format for the content "hello"
        let obj = Object::new(ObjectKind::Blob, b"hello".to_vec());
        let bytes = obj.to_store_bytes();
        assert_eq!(&bytes, b"blob 5\0hello");
    }

    #[test]
    fn blob_known_hash() {
        // git hash-object on "hello\n" produces this SHA1
        let obj = Object::new(ObjectKind::Blob, b"hello\n".to_vec());
        assert_eq!(
            obj.id().to_string(),
            "ce013625030ba8dba906f756967f9e9ca394464a"
        );
    }

    #[test]
    fn write_then_read() {
        let (_dir, repo) = make_repo();
        let store = Store::new(&repo);

        let obj = Object::new(ObjectKind::Blob, b"hello\n".to_vec());
        let id = store.write(&obj).unwrap();

        let got = store.read(&id).unwrap();
        assert_eq!(got.kind, ObjectKind::Blob);
        assert_eq!(got.data, b"hello\n");
    }

    #[test]
    fn write_is_idempotent() {
        let (_dir, repo) = make_repo();
        let store = Store::new(&repo);
        let obj = Object::new(ObjectKind::Blob, b"hello\n".to_vec());

        let id1 = store.write(&obj).unwrap();
        let id2 = store.write(&obj).unwrap();
        assert_eq!(id1, id2);
    }
}
