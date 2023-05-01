use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use bloomfilter::Bloom;
use anyhow::Result;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use glob::glob;

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ShortLink {
    pub slink: String,
    pub dest: String,
}
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-.*!()$~";
const SLINK_LEN: usize = 6;
const NUM_ITEMS: usize = 100_000_000;
const FP_RATE: f64 = 0.00001;
pub fn generate_short_link_batch(save_path: &Path) -> Result<()> {

    let mut bloom = Bloom::new_for_fp_rate(NUM_ITEMS, FP_RATE);
    let pattern = "./*.txt";
    let xx = glob(pattern)?;
    let blooms: Vec<Bloom<String>> = xx.filter_map(|x| x.ok())
        .map(|x| {
            let file = File::open(x)?;
            let mut reader = BufReader::new(file);
            let bloom_bytes_len = reader.read_u64::<LittleEndian>()?;
            let mut bloom_bytes = vec![0u8; bloom_bytes_len as usize];
            reader.read_exact(&mut bloom_bytes)?;
            let bloom: Bloom<String> = serde_json::from_slice(&bloom_bytes)?;
            Ok(bloom)
        }).filter_map(|x: Result<Bloom<String>>| x.ok()).collect();



    let mut unique_links: HashSet<String> = HashSet::new();
    while unique_links.len() < NUM_ITEMS {
        println!("unique_links.len() = {}", unique_links.len());
        let slink = generate_short_link();
        let mut possible_dup = false;
        for b in &blooms {
            if b.check(&slink) {
                possible_dup = true;
                break;
            }
        }
        if possible_dup {
            continue;
        }
        unique_links.insert(slink.clone());
        bloom.set(&slink);
    }

    let file = File::create(save_path)?;
    let mut writer = BufWriter::new(file);
    //serde serialize the bloom
    let bloom_bytes = serde_json::to_vec(&bloom)?;
    let bloom_bytes_len = bloom_bytes.len() as u64;

    writer.write_u64::<LittleEndian>(bloom_bytes_len)?;
    writer.write_all(&bloom_bytes)?;
    for s in &unique_links {
        writer.write_all(s.as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    Ok(())
}

pub fn generate_short_link() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    let string = (0..SLINK_LEN)
        .filter_map(|_| CHARSET.get(rng.gen_range(0..CHARSET.len())))
        .map(|&c| c as char)
        .collect::<String>();
    string


}

impl ShortLink {
    pub fn new(slink: String, dest: String) -> Self {
        Self { slink, dest }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use byteorder::{LittleEndian, ReadBytesExt};
    use crate::domain::short_link::{generate_short_link, ShortLink};

    #[test]
    fn test_new() {
        let random_string = generate_short_link();
        let slink = ShortLink::new(random_string, "https://www.google.com".to_string());
        assert_eq!(slink.slink.len(), 6);
        assert_eq!(slink.dest, "https://www.google.com");
    }
    #[test]
    fn test_generate_short_link_batch() {
        use std::path::Path;
        let save_path = Path::new("test2.txt");
        let _ = super::generate_short_link_batch(save_path);


        let mut file = File::open(save_path).unwrap();
        let length = file.read_u64::<LittleEndian>().unwrap();
        let mut buffer = vec![0; length as usize];
        file.read_exact(&mut buffer).unwrap();

        let bloom = serde_json::from_slice::<bloomfilter::Bloom<String>>(&buffer).unwrap();
        let check = bloom.check(&String::from("X7g19f"));
        assert_eq!(check, false);






    }
}