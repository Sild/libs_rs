use crate::pool_object::PoolObject;
use std::sync::{Arc, Weak};
use tokio::sync::{Mutex, RwLock};

pub struct Pool<T: Send> {
    objects: Arc<Mutex<Vec<T>>>,
    max_size: usize,
    self_ptr: RwLock<Option<Weak<Self>>>,
}

impl<T: Send + 'static> Pool<T> {
    pub async fn new(max_size: usize) -> Arc<Self> {
        let pool = Self {
            objects: Arc::new(Mutex::new(Vec::new())),
            max_size,
            self_ptr: RwLock::new(None),
        };
        let pool_ptr = Arc::new(pool);
        *pool_ptr.self_ptr.write().await = Some(Arc::downgrade(&pool_ptr));
        pool_ptr
    }

    pub async fn try_take_or_create<F>(&self, try_create_fn: F) -> anyhow::Result<PoolObject<T>>
    where
        F: Fn() -> anyhow::Result<T>,
    {
        let inner = {
            let mut lock = self.objects.lock().await;
            if lock.is_empty() {
                try_create_fn()?
            } else {
                lock.pop().unwrap()
            }
        };
        let pool_ptr = self.self_ptr.read().await.as_ref().unwrap().clone();
        Ok(PoolObject::new(inner, pool_ptr))
    }

    pub async fn take_or_create<F>(&self, creator_fn: F) -> PoolObject<T>
    where
        F: Fn() -> T,
    {
        let try_create_fn = || Ok(creator_fn());
        self.try_take_or_create(try_create_fn).await.unwrap()
    }

    /// Put an item back into the pool
    /// Return true if the item was successfully put back into the pool
    /// Return false if the pool is full
    pub(crate) async fn put(&self, item: T) -> bool {
        let mut lock = self.objects.lock().await;
        if lock.len() >= self.max_size {
            return false; // item is dropped
        }
        lock.push(item);
        true
    }
}
