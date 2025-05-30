#![warn(missing_docs)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", env!("CARGO_PKG_README")))]

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::task::{Context, Poll, Waker};

/// Creates a new promise and resolver pair.
pub fn channel<T>() -> (Resolve<T>, Promise<T>) {
    let inner = Arc::new(Inner::new());
    (
        Resolve {
            inner: Some(Arc::downgrade(&inner)),
        },
        Promise { inner },
    )
}

/// A container for a value of type `T` that may not yet be resolved.
///
/// Multiple consumers may obtain the value via shared reference (`&`) once resolved.
pub struct Promise<T> {
    inner: Arc<Inner<T>>,
}
impl<T> Promise<T> {
    /// Waits for the promise to be resolved, returning the value once set.
    ///
    /// Returns `None` if the resolver was dropped before sending a value.
    pub async fn wait(&self) -> Option<&T> {
        std::future::poll_fn(|cx| self.inner.poll_get(cx)).await
    }

    /// Attempts to get the value if it has been resolved, returning an error if not.
    ///
    /// Returns [`PromiseError::Empty`] if the value has not yet been set, and [`PromiseError::Dropped`] if the resolver
    /// was dropped before sending a value.
    pub fn try_get(&self) -> Result<&T, PromiseError> {
        self.inner
            .get()
            .ok_or(PromiseError::Empty)
            .and_then(|value_opt| value_opt.ok_or(PromiseError::Dropped))
    }
}

/// An error that may occur when trying to get the value from a [`Promise`], see [`Promise::try_get`].
#[derive(Debug, Eq, PartialEq, Clone, thiserror::Error)]
pub enum PromiseError {
    /// The resolver has not yet sent a value.
    #[error("value not yet sent")]
    Empty,
    /// The resolver was dropped before sending a value.
    #[error("closed before a value was sent")]
    Dropped,
}

// SAFETY: must not be clonable, as that would allow simultaneous write access to the `Inner` value.
/// The resolve/send half of a promise. Created via [`channel`], and used to resolve the corresponding [`Promise`] with
/// a value.
pub struct Resolve<T> {
    inner: Option<Weak<Inner<T>>>,
}
impl<T> Resolve<T> {
    /// Resolves the promise with the given value, consuming this `Resolve` in the process.
    ///
    /// This will panic if the promise has already been resolved.
    pub fn into_resolve(mut self, value: T) {
        self.resolve(value).unwrap_or_else(|_| panic!("already resolved"));
    }

    /// Resolves the promise with the given value, returning an error if the promise has already been resolved.
    pub fn resolve(&mut self, value: T) -> Result<(), T> {
        let Some(inner) = self.inner.take() else {
            return Err(value);
        };

        if let Some(inner) = inner.upgrade() {
            // SAFETY: `&mut self: Resolve` has exclusive access to `resolve` once.
            unsafe {
                inner.resolve(Some(value));
            }
        }
        Ok(())
    }
}
impl<T> Drop for Resolve<T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take().and_then(|weak| weak.upgrade()) {
            // SAFETY: `&mut self: Resolve` has exclusive access to call resolve once. Because we use `inner.take()`, we
            // know `Self::resolve` was not called.
            unsafe {
                inner.resolve(None);
            }
        }
    }
}

const BIT: u8 = 0b1;
/// Flag for when the [`Resolve`] will no longer change the value of the [`Inner`], because it was either resolved or
/// dropped.
const FLAG_COMPLETED: u8 = BIT;
/// Flag for when the value is set and can be read.
const FLAG_VALUE_SET: u8 = BIT << 1;

/// # Safety
///
/// Any thread with an `&self` may access the `value` field according the following rules:
///
///  1. Iff NOT `FLAG_COMPLETED`, the `value` field may be initialized by the owning `Resolve` once.
///  2. Iff `FLAG_VALUE_SET`, the `value` field may be accessed immutably by any thread.
///
/// If `FLAG_COMPLETED` but NOT `FLAG_VALUE_SET`, then the owning `Resolve` was dropped before the value was set.
///
/// # Table of possible states
/// |                       | NOT `FLAG_COMPLETED`  | `FLAG_COMPLETED`                   |
/// |-----------------------|-----------------------|------------------------------------|
/// | NOT `FLAG_VALUE_SET`  | Waiting for `Resolve` | `Resolve` dropped, `value` not set |
/// | `FLAG_VALUE_SET`      | INVALID               | `value` set, accessible            |
struct Inner<T> {
    flag: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
    /// List of wakers waiting for the value to be set.
    wakers: Mutex<Vec<Waker>>,
}
impl<T> Inner<T> {
    const fn new() -> Self {
        Self {
            flag: AtomicU8::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            wakers: Mutex::new(Vec::new()),
        }
    }

    /// Polls the promise, storing the `Waker` if needed.
    fn poll_get<'a>(&'a self, cx: &mut Context<'_>) -> Poll<Option<&'a T>> {
        if let Some(value_opt) = self.get() {
            return Poll::Ready(value_opt);
        }
        {
            // Acquire lock.
            let mut wakers = self.wakers.lock().unwrap();
            // Check again in case of race condition with resolver.
            if let Some(value_opt) = self.get() {
                return Poll::Ready(value_opt);
            }
            // Add the current waker to the list of wakers.
            wakers.push(cx.waker().clone());
        }
        Poll::Pending
    }

    /// # Returns
    /// * `None` if the value is not yet set.
    /// * `Some(None)` if the resolver was dropped before setting a value.
    /// * `Some(Some(&T))` if the value has been set.
    fn get(&self) -> Option<Option<&T>> {
        // Using acquire ordering so any threads that read a true from this
        // atomic is able to read the value.
        let flag = self.flag.load(Ordering::Acquire);
        let completed = 0 != (flag & FLAG_COMPLETED);
        if completed {
            let value_set = 0 != (flag & FLAG_VALUE_SET);
            if value_set {
                // SAFETY: Value is initialized.
                Some(Some(unsafe { &*(*self.value.get()).as_ptr() }))
            } else {
                // The resolver was dropped before setting a value.
                Some(None)
            }
        } else {
            // The value is not yet set.
            None
        }
    }

    /// SAFETY: The owning [`Resolve`] may call this once.
    unsafe fn resolve(&self, value_or_dropped: Option<T>) {
        let flag = if let Some(value) = value_or_dropped {
            // SAFETY: `&mut self: Resolve` has exclusive access to set the value once.
            unsafe {
                self.value.get().write(MaybeUninit::new(value));
            }
            FLAG_COMPLETED | FLAG_VALUE_SET
        } else {
            // Dropped, do not set `FLAG_INITIALIZED`.
            FLAG_COMPLETED
        };
        // Using release ordering so any threads that read a true from this
        // atomic is able to read the value we just stored.
        self.flag.store(flag, Ordering::Release);
        let wakers = { std::mem::take(&mut *self.wakers.lock().unwrap()) };
        wakers.into_iter().for_each(Waker::wake);
    }
}

/// Since we can get immutable references to [`Self::value`], this is only `Sync` if `T` is `Sync`, otherwise this
/// would allow sharing references of `!Sync` values across threads. We need `T` to be `Send` in order for this to be
/// `Sync` because we can use [`Self::resolve`] to send values (of type T) across threads.
unsafe impl<T: Sync + Send> Sync for Inner<T> {}

/// Access to [`Self::value`] is guarded by the atomic operations on [`Self::flag`], so as long as `T` itself is `Send`
/// it's safe to send it to another thread
unsafe impl<T: Send> Send for Inner<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic() {
        let (resolve, promise) = channel::<i32>();

        let handle = tokio::task::spawn(async move {
            resolve.into_resolve(42);
        });

        let value = promise.wait().await;
        assert_eq!(Some(&42), value);

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple() {
        let (resolve, promise) = channel::<i32>();
        let promise1 = Arc::new(promise);
        let promise2 = Arc::clone(&promise1);
        let promise3 = Arc::clone(&promise1);

        let read1 = tokio::task::spawn(async move {
            let value = promise1.wait().await;
            assert_eq!(Some(&42), value);
        });
        let read2 = tokio::task::spawn(async move {
            let value = promise2.wait().await;
            assert_eq!(Some(&42), value);
        });
        let read3 = tokio::task::spawn(async move {
            let value = promise3.wait().await;
            assert_eq!(Some(&42), value);
        });
        let resolve = tokio::task::spawn(async move {
            resolve.into_resolve(42);
        });

        read1.await.unwrap();
        read2.await.unwrap();
        read3.await.unwrap();
        resolve.await.unwrap();
    }

    #[tokio::test]
    async fn test_try_get() {
        let (resolve, promise) = channel::<i32>();

        assert_eq!(promise.try_get(), Err(PromiseError::Empty));

        resolve.into_resolve(42);

        assert_eq!(promise.try_get(), Ok(&42));
    }

    #[tokio::test]
    async fn test_dropped() {
        let (resolve, promise) = channel::<i32>();

        // Drop the resolver without resolving it.
        drop(resolve);

        // The promise should return an error when trying to get the value.
        assert_eq!(promise.try_get(), Err(PromiseError::Dropped));

        // The promise should return None when waiting for the value.
        assert_eq!(promise.wait().await, None);
    }
}
