wit_bindgen::generate!({
    generate_all
});

use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use exports::wasi::keyvalue::store::{BucketBorrow, Guest, GuestBucket};
use exports::wasi::keyvalue::atomics;
use exports::wasi::keyvalue::store::{self, KeyResponse};

type Result<T> = core::result::Result<T, store::Error>;


fn map_lock<'a, T, U>(
    lock: &'a RwLock<T>,
    f: impl FnOnce(&'a RwLock<T>) -> core::result::Result<U, PoisonError<U>>,
) -> Result<U> {
    f(lock).map_err(|err| store::Error::Other(err.to_string()))
}

pub struct KvBucket {
    buckets: Arc<RwLock<wasi::keyvalue::store::Bucket>>,
}

impl KvBucket {
    fn read(&self) -> Result<RwLockReadGuard<'_, wasi::keyvalue::store::Bucket>> {
        map_lock(&self.buckets, RwLock::read)
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, wasi::keyvalue::store::Bucket>> {
        map_lock(&self.buckets, RwLock::write)
    }
}

impl GuestBucket for KvBucket {
    fn get(&self, key: String) -> Result<Option<Vec<u8>>> {
        let bucket = self.read()?;
        bucket.get(&key)
            .map_err(|err| store::Error::Other(err.to_string()))
    }

    fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        let bucket = self.write()?;
        bucket.set(&key, &value)
            .map_err(|err| store::Error::Other(err.to_string()))
    }

    fn delete(&self, key: String) -> Result<()> {
        let mut bucket = self.write()?;
        bucket.delete(&key)
        .map_err(|err| store::Error::Other(err.to_string()))
    }

    fn exists(&self, key: String) -> Result<bool> {
        let bucket = self.read()?;
        bucket.exists(&key)
            .map_err(|err| store::Error::Other(err.to_string()))
    }

    fn list_keys(&self, cursor: Option<u64>) -> Result<KeyResponse> {
        let bucket = self.read()?;
        bucket.list_keys(cursor)
            .map(| keys| KeyResponse { keys: keys.keys, cursor: keys.cursor })
            .map_err(|err| store::Error::Other(err.to_string()))
    }
}

struct Handler;

impl Guest for Handler {
    type Bucket = KvBucket;

    fn open(identifier: String) -> Result<store::Bucket> {
        let bucket = wasi::keyvalue::store::open("")
            .expect("error opening bucket");
        Ok(store::Bucket::new(KvBucket {
            buckets: Arc::new(RwLock::new(bucket)),
        }))
    }
}

impl atomics::Guest for Handler {
    fn increment(bucket: BucketBorrow<'_>, key: String, delta: u64) -> Result<u64> {
        todo!()
    }
}

export!(Handler);