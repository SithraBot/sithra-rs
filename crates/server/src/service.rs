use std::task::{Context, Poll};

use futures_util::future::{JoinAll, join_all};
use pin_project::pin_project;
use tower::Service;

use crate::response::{IntoResponse, Response};

#[derive(Clone, Debug)]
pub struct JoinAllService<S, const N: usize> {
    inner: [S; N],
}

impl<S, const N: usize> JoinAllService<S, N> {
    pub const fn new(inner: [S; N]) -> Self {
        Self { inner }
    }
}

impl<S, const N: usize, Request, Error, Fut> Service<Request> for JoinAllService<S, N>
where
    Request: Clone,
    Error: Send + 'static,
    Fut: Future<Output = Result<Response, Error>> + Send + 'static,
    S: Service<Request, Response = Response, Error = Error, Future = Fut>,
{
    type Error = Error;
    type Future = JoinAllServiceFuture<S::Future>;
    type Response = Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.inner {
            match service.poll_ready(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let futures: [_; N] = std::array::from_fn(|i| self.inner[i].call(req.clone()));
        JoinAllServiceFuture::new(futures)
    }
}

#[pin_project]
pub struct JoinAllServiceFuture<Fut: Future> {
    #[pin]
    inner: JoinAll<Fut>,
}

impl<Fut: Future> JoinAllServiceFuture<Fut> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Fut>,
    {
        Self {
            inner: join_all(iter),
        }
    }
}

impl<Error, Fut> Future for JoinAllServiceFuture<Fut>
where
    Error: Send + 'static,
    Fut: Future<Output = Result<Response, Error>>,
{
    type Output = Result<Response, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx).map(|mut r| {
            let is_all_err = {
                let mut r = r.iter();
                r.all(Result::is_err)
            };
            if is_all_err {
                r.pop().unwrap_or_else(|| Ok(Response::none()))
            } else {
                Ok(r.into_iter()
                    .filter_map(|v| {
                        let Ok(v) = v else { return None };
                        Some(v)
                    })
                    .collect::<Vec<_>>()
                    .into_response())
            }
        })
    }
}
