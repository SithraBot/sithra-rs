use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use pin_project::pin_project;
use sithra_transport::channel::Channel;
use tower::{
    Layer, Service, ServiceExt,
    util::{BoxCloneSyncService, MapErrLayer, Oneshot},
};
use ulid::Ulid;

use crate::{
    request::Request,
    response::{IntoResponse, MapIntoResponse, Response},
};

#[derive(Clone, Debug)]
pub struct Route<E = Infallible>(BoxCloneSyncService<Request, Response, E>);

impl<E> Route<E> {
    pub(crate) fn new<T>(svc: T) -> Self
    where
        T: Service<Request, Error = E> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse + 'static,
        T::Future: Send + 'static,
    {
        Self(BoxCloneSyncService::new(MapIntoResponse::new(svc)))
    }

    /// Variant of [`Route::call`] that takes ownership of the route to avoid
    /// cloning.
    pub(crate) fn call_owned(self, req: Request) -> RouteFuture<E> {
        self.oneshot_inner_owned(req)
    }

    pub(crate) fn oneshot_inner(&self, req: Request) -> RouteFuture<E> {
        let correlation = req.correlation();
        let channel = req.channel();
        let bot_id = req.bot_id();
        RouteFuture::new(self.0.clone().oneshot(req), correlation, channel, bot_id)
    }

    /// Variant of [`Route::oneshot_inner`] that takes ownership of the route to
    /// avoid cloning.
    pub(crate) fn oneshot_inner_owned(self, req: Request) -> RouteFuture<E> {
        let correlation = req.correlation();
        let channel = req.channel();
        let bot_id = req.bot_id();
        RouteFuture::new(self.0.oneshot(req), correlation, channel, bot_id)
    }

    pub(crate) fn layer<L, NewError>(self, layer: L) -> Route<NewError>
    where
        L: Layer<Self> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<NewError> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
        NewError: 'static,
    {
        let layer = (MapErrLayer::new(Into::into), layer);

        Route::new(layer.layer(self))
    }
}

impl<E> Service<Request> for Route<E> {
    type Error = E;
    type Future = RouteFuture<E>;
    type Response = Response;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: Request) -> Self::Future {
        self.oneshot_inner(req)
    }
}

#[pin_project(project = RouteFutureProj)]
pub enum RouteFuture<E = Infallible> {
    Oneshot(#[pin] RouteFutureOneshot<E>),
    Ready(Option<Response>),
}

#[pin_project]
pub struct RouteFutureOneshot<E> {
    #[pin]
    inner:       Oneshot<BoxCloneSyncService<Request, Response, E>, Request>,
    correlation: Ulid,
    channel:     Option<Channel>,
    bot_id:      Option<String>,
}

impl<E> RouteFuture<E> {
    const fn new(
        inner: Oneshot<BoxCloneSyncService<Request, Response, E>, Request>,
        correlation: Ulid,
        channel_opt: Option<Channel>,
        bot_id_opt: Option<String>,
    ) -> Self {
        Self::Oneshot(RouteFutureOneshot {
            inner,
            correlation,
            channel: channel_opt,
            bot_id: bot_id_opt,
        })
    }

    #[must_use]
    pub const fn ready(response: Response) -> Self {
        Self::Ready(Some(response))
    }
}

impl<E> Future for RouteFuture<E> {
    type Output = Result<Response, E>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            RouteFutureProj::Oneshot(route_future_oneshot) => {
                let this = route_future_oneshot.project();
                let mut res = ready!(this.inner.poll(cx))?;
                res.correlate(*this.correlation);
                if let Some(channel) = this.channel.take() {
                    res.set_channel(&channel);
                }
                if let Some(bot_id) = this.bot_id.take() {
                    res.set_bot_id(bot_id);
                }
                Poll::Ready(Ok(res))
            }
            RouteFutureProj::Ready(response) => {
                Poll::Ready(Ok(response.take().expect("unreachable")))
            }
        }
    }
}
