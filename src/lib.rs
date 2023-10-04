#![feature(pointer_byte_offsets)]

#[cfg(test)]
mod test;
mod traits;
pub use traits::GHash;

use std::{
    fmt::Debug,
    marker::PhantomData,
    borrow::Borrow,
};


use dyn_array::Array;
use int_ll::{
    LinkedListAnchor,
    LinkedList,
};
use blake3::{
    Hash,
    Hasher
};
use static_assertions::const_assert;

#[derive(PartialEq, Eq)]
struct Anon {}
impl GHash for Anon {
    fn hash(&self, _:&mut Hasher) -> Hash {
        todo!("should not be called");
    }
}

#[derive(Debug)]
struct KeyStruct<KeyType:GHash+Eq> {
    hash: Hash,
    content_index: usize,
    anchor: LinkedListAnchor,
    key:KeyType,
}

#[derive(Debug)]
struct NodeBases {
    keys: *mut u8,
    contents: *mut u8, 
}

const_assert!(core::mem::size_of::<NodeBases>() < core::mem::size_of::<KeyStruct<Anon>>());

const KEY_ANCHOR_OFFSET:usize = memoffset::offset_of!(KeyStruct<Anon>, anchor);

#[derive(Debug)]
pub struct HashMap<KeyType:GHash+Eq, ContentType> {
    
    phantom_key: PhantomData<KeyType>,
    phantom_content: PhantomData<ContentType>,
    
    buckets: usize,
    len: usize,
    buckets_array: Array<LinkedList<KEY_ANCHOR_OFFSET, KeyStruct<KeyType>>>,
    keys: Array<KeyStruct<KeyType>>,
    contents: Array<ContentType>,
}



impl<KeyType:GHash+Eq, ContentType> HashMap<KeyType, ContentType> {
    
    pub fn new() -> Self {
        let initial_buckets = 64;
        let mut buckets_array:Array<LinkedList<KEY_ANCHOR_OFFSET, KeyStruct<KeyType>>> = Array::new().unwrap();
        
        let keys:Array<KeyStruct<KeyType>> = Array::new().unwrap();
        let contents:Array<ContentType> = Array::new().unwrap();
        
        let (_, memory) = buckets_array.allocate().unwrap();
        let cast = memory.as_ptr() as *mut NodeBases;
        let cast_mut = unsafe{cast.as_mut().expect("should exist")};
        
        cast_mut.keys = keys.base().as_ptr() as *mut u8;
        cast_mut.contents = contents.base().as_ptr() as *mut u8;
        
        for _elem in 0..initial_buckets {
            let (_, mut bucket_memory) = buckets_array.allocate().unwrap();
            let bucket_mut = unsafe{bucket_memory.as_mut()};
            bucket_mut.remplaze_new_extern(std::ptr::addr_of_mut!(cast_mut.keys));
        }
        
        Self{
            len: 0,
            buckets: initial_buckets,
            buckets_array,
            phantom_key: PhantomData,
            phantom_content: PhantomData,
            keys,
            contents,
        }
    }
    
    pub fn insert(&mut self, key:KeyType, contents:ContentType) -> Result<(), ()> {
        let mut hasher = Hasher::new();
        let key_hash = key.hash(&mut hasher);
        
        let hash_usize = usize::from_be_bytes(key_hash.as_bytes()[0..8].try_into().expect("hard codded values, should not crash"));
        let bucket_index = hash_usize%self.buckets;
        
        let _node_bases = self.get_node_bases();
        /*
        println!("{hash_usize:X}");
        println!("{bucket_index}");
        println!("{node_bases:?}");
        */
        
        let (new_addr, mut key_memory) = self.keys.allocate().expect("allocator should not fail");
        
        if let Some(_) = new_addr {
            todo!("address changed");
        }
        
        let key_mut = unsafe{key_memory.as_mut()};
        let content_index = self.contents.len();
        
        *key_mut = KeyStruct{
            hash: key_hash,
            content_index: content_index,
            anchor: LinkedListAnchor::default(),
            key: key,
        };
        
        let bucket_holder = &mut self.buckets_array[bucket_index+1];
        bucket_holder.insert_mem(key_memory).unwrap();
        
        let (new_addr, mut content_memory) = self.contents.allocate().expect("allocator should not fail");
        
        if let Some(_) = new_addr {
            todo!("address changed");
        }
        let content_mut = unsafe{content_memory.as_mut()};
        
        *content_mut = contents;
        
        self.len += 1;
        Ok(())
    }
    
    fn get_node_bases(&mut self) -> &mut NodeBases {
        let holder = self.buckets_array.get_ptr_mut(0);
        let holder = holder as *mut NodeBases;
        unsafe{holder.as_mut().unwrap()}
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    

    pub fn get<KeySearchType>(&self, key:&KeySearchType) -> Option<&ContentType>
    where 
        KeyType:Borrow<KeySearchType>,
        KeySearchType:?Sized+Eq+GHash,
    {
        let mut hasher = Hasher::new();
        let key_hash = key.hash(&mut hasher);
        
        let hash_usize = usize::from_be_bytes(key_hash.as_bytes()[0..8].try_into().expect("hard codded values, should not crash"));
        let bucket_index = hash_usize%self.buckets;
        
        let mut index = None;
        for elem in self.buckets_array[bucket_index+1].iter() {
            let elem_ref = unsafe{elem.as_ref().unwrap()};
            if elem_ref.hash.as_bytes() == key_hash.as_bytes()  {
                if elem_ref.key.borrow() == key {
                    index = Some(elem_ref.content_index);
                    break;
                }
            }
        }
        let found_index = index?;
        
        Some(&self.contents[found_index])
    }
}


