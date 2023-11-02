use crate::generated;

#[derive(Debug)]
pub struct Pool(*mut generated::apr_pool_t);

impl From<Pool> for *mut generated::apr_pool_t {
    fn from(p: Pool) -> Self {
        p.0
    }
}

impl Pool {
    /// Create a new pool.
    pub fn new() -> Self {
        let mut pool: *mut generated::apr_pool_t = std::ptr::null_mut();
        unsafe {
            generated::apr_pool_create_ex(
                &mut pool,
                std::ptr::null_mut(),
                None,
                std::ptr::null_mut() as *mut generated::apr_allocator_t,
            );
        }
        Pool(pool)
    }

    /// Create a subpool.
    pub fn subpool(&mut self) -> Self {
        let mut subpool: *mut generated::apr_pool_t = std::ptr::null_mut();
        unsafe {
            generated::apr_pool_create_ex(
                &mut subpool,
                self.0,
                None,
                std::ptr::null_mut() as *mut generated::apr_allocator_t,
            );
        }
        Pool(subpool)
    }

    /// Allocate memory in the pool.
    pub fn alloc<T: Sized>(&mut self) -> *mut std::mem::MaybeUninit<T> {
        let size = std::mem::size_of::<T>();
        unsafe { generated::apr_palloc(self.0, size) as *mut std::mem::MaybeUninit<T> }
    }

    /// Allocate memory in the pool, and initialize it to zero.
    pub fn calloc<T: Sized>(&mut self) -> *mut T {
        let data = self.alloc::<T>();
        unsafe {
            std::ptr::write_bytes(data as *mut u8, 0, std::mem::size_of::<T>());
        }
        data as *mut T
    }

    /// Check if the pool is an ancestor of another pool.
    pub fn is_ancestor(&self, other: &Pool) -> bool {
        unsafe { generated::apr_pool_is_ancestor(self.0, other.0) != 0 }
    }

    /// Set a tag for the pool.
    pub fn tag(&self, tag: &str) {
        unsafe {
            generated::apr_pool_tag(self.0, tag.as_ptr() as *const i8);
        }
    }

    /// Clear all memory in the pool.
    ///
    /// This does not actually free the memory, it just allows the pool to reuse this memory for the next allocation.
    ///
    /// # Safety
    ///
    /// This is unsafe because it is possible to create a dangling pointer to memory that has been cleared.
    pub unsafe fn clear(&mut self) {
        unsafe {
            generated::apr_pool_clear(self.0);
        }
    }

    /// Get the parent pool, if any.
    pub fn parent(&self) -> Self {
        let parent = unsafe { generated::apr_pool_parent_get(self.0) };
        Pool(parent)
    }
}

impl From<&mut Pool> for *mut generated::apr_pool_t {
    fn from(p: &mut Pool) -> Self {
        p.0
    }
}

impl From<&Pool> for *mut generated::apr_pool_t {
    fn from(p: &Pool) -> Self {
        p.0
    }
}

impl Default for Pool {
    fn default() -> Self {
        Pool::new()
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            generated::apr_pool_destroy(self.0);
        }
    }
}
pub struct Allocator(*mut generated::apr_allocator_t);

impl From<Allocator> for *mut generated::apr_allocator_t {
    fn from(a: Allocator) -> Self {
        a.0
    }
}

impl Allocator {
    pub fn new() -> Self {
        let mut allocator: *mut generated::apr_allocator_t = std::ptr::null_mut();
        unsafe {
            generated::apr_allocator_create(&mut allocator);
        }
        Allocator(allocator)
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Allocator::new()
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        unsafe {
            generated::apr_allocator_destroy(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool() {
        let mut pool = Pool::new();
        let subpool = pool.subpool();
        assert!(pool.is_ancestor(&subpool));
        assert!(!subpool.is_ancestor(&pool));
        assert!(subpool.parent().is_ancestor(&subpool));
        subpool.tag("subpool");
        pool.tag("pool");
    }
}

/// A wrapper around a value that is allocated in a pool.
pub struct Pooled<'pool, T>
where
    T: 'pool,
{
    pool: std::rc::Rc<Pool>,
    data: T,
    _marker: std::marker::PhantomData<&'pool T>,
}

impl<'pool, T> Pooled<'pool, T> {
    /// Create a pooled value, using the given closure to initialize it.
    pub fn initialize<E: std::error::Error>(
        cb: impl FnOnce(&mut Pool) -> Result<T, E>,
    ) -> Result<Self, E> {
        let mut pool = std::rc::Rc::new(Pool::new());
        let data = cb(std::rc::Rc::get_mut(&mut pool).as_mut().unwrap())?;
        Ok(Pooled {
            pool,
            data,
            _marker: std::marker::PhantomData,
        })
    }

    /// Create a pooled value from a value allocated in a pool.
    ///
    /// # Safety
    ///
    /// The data must be allocated in the pool.
    pub unsafe fn in_pool(pool: std::rc::Rc<Pool>, data: T) -> Self {
        // Assert that the data is allocated in the pool.
        Pooled {
            pool,
            data,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get a reference to the pool that the value is allocated in.
    pub fn pool(&self) -> std::rc::Rc<Pool> {
        self.pool.clone()
    }
}

impl<'pool, T> AsRef<T> for Pooled<'pool, T> {
    fn as_ref(&self) -> &T {
        &self.data
    }
}

impl<'pool, T> std::ops::Deref for Pooled<'pool, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'pool, T> std::ops::DerefMut for Pooled<'pool, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A wrapper around a pointer to a value that is allocated in a pool.
pub struct PooledPtr<'pool, T>
where
    T: 'pool,
{
    pool: std::rc::Rc<Pool>,
    data: *mut T,
    _marker: std::marker::PhantomData<&'pool T>,
}

impl<'pool, T> PooledPtr<'pool, T> {
    /// Create a pooled value, using the given closure to initialize it.
    pub fn initialize<E: std::error::Error>(
        cb: impl FnOnce(&mut Pool) -> Result<*mut T, E>,
    ) -> Result<Self, E> {
        let mut pool = std::rc::Rc::new(Pool::new());
        let data = cb(std::rc::Rc::get_mut(&mut pool).as_mut().unwrap())?;
        Ok(PooledPtr {
            pool,
            data,
            _marker: std::marker::PhantomData,
        })
    }

    /// Create a pooled value from a value allocated in a pool.
    ///
    /// # Safety
    ///
    /// The data must be allocated in the pool.
    pub unsafe fn in_pool(pool: std::rc::Rc<Pool>, data: *mut T) -> Self {
        // TODO: Assert that the data is allocated in the pool.
        PooledPtr {
            pool,
            data,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// Get a reference to the pool that the value is allocated in.
    pub fn pool(&self) -> std::rc::Rc<Pool> {
        self.pool.clone()
    }
}

impl<'pool, T> AsRef<T> for PooledPtr<'pool, T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.data }
    }
}
impl<'pool, T> std::ops::Deref for PooledPtr<'pool, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'pool, T> std::ops::DerefMut for PooledPtr<'pool, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}