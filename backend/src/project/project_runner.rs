use actix::{Actor, ActorFutureExt, AsyncContext, WrapFuture};
use tokio::sync::oneshot;

pub type AbortSender = oneshot::Sender<()>;
pub type AbortReceiver = oneshot::Receiver<()>;

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
