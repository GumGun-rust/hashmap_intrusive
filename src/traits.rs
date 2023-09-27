use blake3::{
    Hash,
    Hasher,
};

pub trait GHash {
    fn hash(&self, hash:Hasher) -> Hash;
}
