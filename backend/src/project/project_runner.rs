use std::sync::Arc;

use actix::{Actor, ActorFutureExt, AsyncContext, WrapFuture};
use tokio::sync::{oneshot, Mutex};

pub type AbortSender = oneshot::Sender<()>;
pub type AbortReceiver = oneshot::Receiver<()>;
pub type SharedAbort = SharedExport<AbortSender>;

#[derive(Debug)]
#[repr(transparent)]
pub struct SharedExport<T>(Arc<Mutex<Option<T>>>);

impl<T> Clone for SharedExport<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Default for SharedExport<T> {
    fn default() -> Self {
        Self(Mutex::new(None).into())
    }
}

impl<T> SharedExport<T> {
    pub async fn set_value(&self, val: T) {
        self.0.lock().await.replace(val);
    }

    pub async fn take_value(&self) -> Option<T> {
        self.0.lock().await.take()
    }

    pub async fn has_value(&self) -> bool {
        self.0.lock().await.is_some()
    }
}

pub fn create_abort() -> (AbortSender, AbortReceiver) {
    oneshot::channel::<()>()
}

pub fn start_job<Next, A, BeforeFn, JobFn, FinishFn>(
    this: &mut A,
    ctx: &mut A::Context,
    before: BeforeFn,
    job: JobFn,
    finish: FinishFn,
) where
    A: Actor,
    A::Context: AsyncContext<A>,
    BeforeFn: 'static + FnOnce(AbortSender, &mut A, &mut A::Context) -> (),
    JobFn: 'static + AsyncFnOnce(AbortReceiver) -> Next,
    FinishFn: 'static + FnOnce(Next, &mut A, &mut A::Context) -> (),
{
    let (abort_sender, abort_recv) = create_abort();

    before(abort_sender, this, ctx);

    ctx.spawn(
        actix::fut::ready(())
            .then(move |_, this, _| job(abort_recv).into_actor(this))
            .map(finish),
    );
}
