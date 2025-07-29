use std::future::Future;

use tokio::task::JoinHandle;

pub trait Optional<T> {
    fn or_default(self) -> T
    where
        T: Default;
}

impl<T> Optional<T> for Option<T> {
    fn or_default(self) -> T
    where
        T: Default,
    {
        self.unwrap_or_default()
    }
}

pub trait OptionalFuture<T: Future> {
    fn as_fut(self) -> impl Future<Output = Option<T::Output>>;
}

impl<T: Future> OptionalFuture<T> for Option<T> {
    async fn as_fut(self) -> Option<T::Output> {
        match self {
            Some(fut) => Some(fut.await),
            None => None,
        }
    }
}

pub trait FutureExt: Future {
    fn or_default<T>(self) -> impl Future<Output = T>
    where
        Self::Output: Optional<T>,
        T: Default;

    fn spawn(self) -> JoinHandle<Self::Output>
    where
        Self: Send + 'static,
        Self::Output: Send + 'static;

    fn wrap_fut<Fn, R>(self, f: Fn) -> impl Future<Output = R>
    where
        Self::Output: Sized,
        Fn: FnOnce(Self::Output) -> R;
}

impl<F: Future> FutureExt for F {
    async fn or_default<T>(self) -> T
    where
        Self::Output: Optional<T>,
        T: Default,
    {
        self.await.or_default()
    }

    fn spawn(self) -> JoinHandle<Self::Output>
    where
        Self: Send + 'static,
        Self::Output: Send + 'static,
    {
        tokio::spawn(self)
    }

    async fn wrap_fut<Fn, R>(self, f: Fn) -> R
    where
        Self::Output: Sized,
        Fn: FnOnce(Self::Output) -> R,
    {
        f(self.await)
    }
}
