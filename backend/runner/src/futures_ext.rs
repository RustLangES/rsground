use std::future::Future;

use tokio::task::JoinHandle;

pub trait OptionalFuture<T: Future> {
    fn as_fut(self) -> impl Future<Output = Option<T::Output>>;
}

impl<T: Future> OptionalFuture<T> for Option<T> {
    async fn as_fut(self) -> Option<T::Output> {
        if let Some(fut) = self {
            Some(fut.await)
        } else {
            None
        }
    }
}

pub trait FutureOption<T> {
    fn or_default(self) -> impl Future<Output = T>
    where
        T: Default;
}

impl<T, F: Future<Output = Option<T>>> FutureOption<T> for F {
    async fn or_default(self) -> T
    where
        T: Default,
    {
        self.await.unwrap_or_default()
    }
}

pub trait FutureExt: Future {
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
