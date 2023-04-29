use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ShortLink {
    pub slink: String,
    pub dest: String,
}

impl ShortLink {
    pub fn new(slink: String, dest: String) -> Self {
        Self { slink, dest }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::short_link::ShortLink;

    #[test]
    fn test_new() {
        use rand::Rng;

        let base64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut rng = rand::thread_rng();

        let random_string: String = (0..6)
            .filter_map(|_| base64_chars.chars().nth(rng.gen_range(0..base64_chars.len())))
            .collect();
        let slink = ShortLink::new(random_string, "https://www.google.com".to_string());
        assert_eq!(slink.slink.len(), 6);
        assert_eq!(slink.dest, "https://www.google.com");
    }
}