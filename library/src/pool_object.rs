use crate::pool::Pool;
use std::ops::{Deref, DerefMut};
use std::sync::Weak;
use tokio::task;

pub struct PoolObject<T: Send + 'static> {
    inner: Option<T>,
    parent: Weak<Pool<T>>,
}

impl<T: Send + 'static> PoolObject<T> {
    pub(crate) fn new(inner: T, parent: Weak<Pool<T>>) -> Self {
        Self {
            inner: Some(inner),
            parent,
        }
    }
}

impl<T: Send + 'static> Deref for PoolObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<T: Send + 'static> DerefMut for PoolObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl<T: Send + 'static> Drop for PoolObject<T> {
    fn drop(&mut self) {
        let inner = self.inner.take().unwrap();
        if let Some(parent) = self.parent.upgrade() {
            task::spawn(async move {
                parent.put(inner).await;
            });
        }
    }
}
