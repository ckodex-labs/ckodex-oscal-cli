use sha2::{Digest, Sha256};

pub fn compute_merkle_root(leaf_digests: &[String]) -> String {
    if leaf_digests.is_empty() {
        return "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            .to_string(); // Empty string sha256
    }
    if leaf_digests.len() == 1 {
        return leaf_digests[0].clone();
    }

    let mut current_layer: Vec<String> = leaf_digests.to_vec();

    while current_layer.len() > 1 {
        let mut next_layer = Vec::new();
        for chunk in current_layer.chunks(2) {
            if chunk.len() == 2 {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0].as_bytes());
                hasher.update(chunk[1].as_bytes());
                let res = format!("sha256:{}", hex::encode(hasher.finalize()));
                next_layer.push(res);
            } else {
                next_layer.push(chunk[0].clone());
            }
        }
        current_layer = next_layer;
    }

    current_layer[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_deterministic() {
        let leaves = vec![
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            "sha256:2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            "sha256:3333333333333333333333333333333333333333333333333333333333333333".to_string(),
        ];
        let root1 = compute_merkle_root(&leaves);
        let root2 = compute_merkle_root(&leaves);
        assert_eq!(root1, root2);
        assert!(root1.starts_with("sha256:"));
    }
}
