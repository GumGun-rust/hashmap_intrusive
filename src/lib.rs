#![feature(pointer_byte_offsets)]

mod traits;
pub use traits::GHash;

use std::marker::PhantomData;

use dyn_array::Array;
use int_ll::{
    LinkedListAnchor,
    LinkedList,
};
use blake3::{
    Hash,
    Hasher
};

struct Anon {}
impl GHash for Anon {
    fn hash(&self, _:Hasher) -> Hash {
        todo!("should not be called");
    }
}

struct KeyStruct<KeyType:GHash> {
    hash: Hash,
    content: usize,
    anchor: LinkedListAnchor,
    key:KeyType,
}



const KEY_ANCHOR_OFFSET:usize = memoffset::offset_of!(KeyStruct<Anon>, anchor);

struct HashMap<KeyType:GHash, ContentType> {
    phantom_key: PhantomData<KeyType>,
    phantom_content: PhantomData<ContentType>,
    
    heads_array: Array<LinkedList<KEY_ANCHOR_OFFSET, KeyStruct<KeyType>>>,
    keys: Array<KeyStruct<KeyType>>,
    content: Array<ContentType>,
}

