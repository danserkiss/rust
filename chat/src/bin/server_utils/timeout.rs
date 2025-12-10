use anyhow::anyhow;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::Duration;

#[pin_project]
pub struct Timeout<F> {
    #[pin]
    future: F,
    #[pin]
    timeout: tokio::time::Sleep,
}

impl<F> Timeout<F> {
    pub fn new(timeout: Duration, future: F) -> Self {
        Timeout {
            timeout: tokio::time::sleep(timeout),
            future: future,
        }
    }
}

impl<F> Future for Timeout<F>
where
    F: Future,
{
    type Output = Result<F::Output, anyhow::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Ready(res) => return Poll::Ready(Ok(res)),
            Poll::Pending => {}
        }
        match this.timeout.poll(cx) {
            Poll::Ready(_) => return Poll::Ready(Err(anyhow!("TimeOut"))),
            Poll::Pending => {}
        }
        Poll::Pending
    }
}
