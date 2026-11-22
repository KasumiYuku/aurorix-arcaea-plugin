
use std::path::Path;
use std::sync::OnceLock;

const POOL_FILE: &str = "jacket_pool.json";

const POOL_VERSION: i32 = 1;

#[derive(serde::Deserialize)]
struct PoolDoc {
    #[serde(default)]
    version: i32,
    #[serde(default)]
    jackets: Vec<String>,
}

fn pool() -> &'static [String] {
    static POOL: OnceLock<Vec<String>> = OnceLock::new();
    POOL.get_or_init(|| {
        let root = crate::assets_root();
        let Ok(text) = std::fs::read_to_string(root.join(POOL_FILE)) else {
            return Vec::new();
        };
        let Ok(doc) = serde_json::from_str::<PoolDoc>(&text) else {
            return Vec::new();
        };
        if doc.version != POOL_VERSION {
            return Vec::new();
        }
        let mut ids: Vec<String> = doc
            .jackets
            .into_iter()
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .filter(|id| has_art(&root, id))
            .collect();
        ids.sort();
        ids.dedup();
        ids
    })
}

fn has_art(root: &Path, id: &str) -> bool {
    root.join("jackets").join(format!("{id}.jpg")).exists()
        || root.join("jacket_thumbs").join(format!("{id}.jpg")).exists()
}

pub fn seed64(parts: &[&str]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
    }
    hash
}

pub fn board_art(parts: &[&str]) -> Option<&'static str> {
    let ids = pool();
    if ids.is_empty() {
        return None;
    }
    Some(&ids[(seed64(parts) % ids.len() as u64) as usize])
}

pub fn ledger_art(parts: &[&str]) -> Option<&'static str> {
    let ids = pool();
    if ids.is_empty() {
        return None;
    }
    Some(&ids[(seed64(parts) % ids.len() as u64) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_096() {
        let a = seed64(&["trend", "12.345", "40"]);
        let b = seed64(&["trend", "12.345", "40"]);
        let c = seed64(&["trend", "12.346", "40"]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn case_097() {
        assert_eq!(seed64(&[]), 14695981039346656037);
    }
}
