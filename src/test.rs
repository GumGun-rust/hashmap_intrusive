
use super::{
    HashMap,
    GHash,
};

use blake3::{
    Hash,
    Hasher
};

#[derive(Debug, PartialEq, Eq)]
struct BigKey{
    pub content: [u64;8],
}

impl GHash for BigKey {
    fn hash(&self, hasher:&mut Hasher) -> Hash {
        hasher.reset();
        
        let mut res = [0u8; 64];
        for i in 0..8 {
            res[4*i..][..8].copy_from_slice(&self.content[i].to_le_bytes());
        }
        hasher.update(&res);
        hasher.finalize()
    }
}

#[test]
fn hash_map() {
    let mut holder:HashMap<BigKey, u32> = HashMap::new();
    for elem in 0..70 {
        holder.insert(BigKey{content:[0,0,0,0,0,0,0,elem.into()]}, elem*16).unwrap();
    }
    let value = holder.get(&BigKey{content:[0,0,0,0,0,0,0,16]});
    
    //println!("{:#?}", holder);
    println!("{value:?}");
}
