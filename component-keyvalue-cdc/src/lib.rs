wit_bindgen::generate!({
    generate_all
});

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

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

#[derive(Clone, Default)]
pub struct KvBucket(Arc<RwLock<HashMap<String, Vec<u8>>>>);

impl KvBucket {
    fn read(&self) -> Result<RwLockReadGuard<'_, HashMap<String, Vec<u8>>>> {
        map_lock(&self.0, RwLock::read)
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, HashMap<String, Vec<u8>>>> {
        map_lock(&self.0, RwLock::write)
    }
}

impl GuestBucket for KvBucket {
    fn get(&self, key: String) -> Result<Option<Vec<u8>>> {
        let bucket = self.read()?;
        Ok(bucket.get(&key).cloned())
    }

    fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        let mut bucket = self.write()?;
        bucket.insert(key, value);
        Ok(())
    }

    fn delete(&self, key: String) -> Result<()> {
        let mut bucket = self.write()?;
        bucket.remove(&key);
        Ok(())
    }

    fn exists(&self, key: String) -> Result<bool> {
        let bucket = self.read()?;
        Ok(bucket.contains_key(&key))
    }

    fn list_keys(&self, cursor: Option<u64>) -> Result<KeyResponse> {
        let bucket = self.read()?;
        let bucket = bucket.keys();
        let keys = if let Some(cursor) = cursor {
            let cursor =
                usize::try_from(cursor).map_err(|err| store::Error::Other(err.to_string()))?;
            bucket.skip(cursor).cloned().collect()
        } else {
            bucket.cloned().collect()
        };
        Ok(KeyResponse { keys, cursor: None })
    }
}

struct Handler;

impl Guest for Handler {
    type Bucket = KvBucket;

    fn open(identifier: String) -> Result<store::Bucket> {
        static STORE: OnceLock<Mutex<HashMap<String, KvBucket>>> = OnceLock::new();
        let store = STORE.get_or_init(Mutex::default);
        let mut store = store.lock().expect("failed to lock store");
        let bucket = store.entry(identifier).or_default().clone();
        Ok(store::Bucket::new(bucket))
    }
}

impl atomics::Guest for Handler {
    fn increment(bucket: BucketBorrow<'_>, key: String, delta: u64) -> Result<u64> {
        todo!()
    }
}

export!(Handler);