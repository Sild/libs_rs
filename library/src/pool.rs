use crate::pool_object::PoolObject;
use std::sync::{Arc, Condvar, Mutex, RwLock, Weak};
use crate::{Config, Error};

pub type ArcPool<T> = Arc<Pool<T>>;

pub struct Pool<T: Send> {
    config: Config,
    storage: Arc<(Mutex<Vec<T>>, Condvar)>,
    self_ptr: RwLock<Option<Weak<Self>>>,
}

impl<T: Send + 'static> Pool<T> {

    pub fn new<I>(items: I) -> Result<ArcPool<T>, Error>
    where
        I: IntoIterator<Item = T>,
    {
        Self::with_config(Config::default(), items)
    }

    pub fn with_config<I>(config: Config, items: I) -> Result<ArcPool<T>, Error>
    where
        I: IntoIterator<Item = T>,
    {
        let objects = items.into_iter().collect();
        let pool = Self {
            config,
            storage: Arc::new((Mutex::new(objects), Condvar::new())),
            self_ptr: RwLock::new(None),
        };
        let pool_ptr = Arc::new(pool);
        *pool_ptr.self_ptr.write()? = Some(Arc::downgrade(&pool_ptr));
        Ok(pool_ptr)
    }

    pub fn take(&self) -> Result<Option<PoolObject<T>>, Error> {
        let (mtx, cvar) = &*self.storage;
        let mut lock = mtx.lock()?;
        while lock.is_empty() {
            let (new_lock, is_timeout) = cvar.wait_timeout(lock, self.config.wait_duration)?;
            if is_timeout.timed_out() {
                return Ok(None);
            }
            lock = new_lock;
        }
        let inner = lock.pop().unwrap();
        drop(lock);
        let pool_ptr = self.self_ptr.read()?.as_ref().unwrap().clone();
        Ok(Some(PoolObject::new(inner, pool_ptr)))
    }

    pub fn size(&self) -> Result<usize, Error> {
        Ok(self.storage.0.lock()?.len())
    }

    pub(crate) fn put(&self, item: T) -> Result<(), Error> {
        self.storage.0.lock()?.push(item);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use crate::Config;
    use crate::pool::{Config, Pool};

    #[test]
    fn test_workflow() -> anyhow::Result<()> {
        let config = Config {
            wait_duration: std::time::Duration::from_millis(5),
        };
        let pool = Pool::with_config(config, [1, 2, 3])?;
        assert_eq!(pool.size()?, 3);

        let obj1 = pool.take()?;
        assert_eq!(pool.size()?, 2);
        assert_eq!(*obj1.as_ref().unwrap().deref(), 3);

        let obj2 = pool.take()?;
        assert_eq!(*obj2.as_ref().unwrap().deref(), 2);
        let obj3 = pool.take()?;
        assert_eq!(pool.size()?, 0);
        assert_eq!(*obj3.as_ref().unwrap().deref(), 1);

        let obj4 = pool.take()?;
        assert!(obj4.is_none());

        Ok(())
    }
}
