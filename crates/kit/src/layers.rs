use std::{
    convert::Infallible,
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use pin_project::pin_project;
use sithra_server::{
    request::Request,
    response::Response,
    routing::route::{Route, RouteFuture},
};
use tower::{Layer, Service};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BotId(pub String);

impl BotId {
    pub fn new(id: impl Display) -> Self {
        Self(id.to_string())
    }
}

impl Layer<Route> for BotId {
    type Service = BotIdLayer;

    fn layer(&self, inner: Route) -> Self::Service {
        BotIdLayer {
            id:  self.0.clone(),
            svc: inner,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BotIdLayer {
    id:  String,
    svc: Route,
}

impl Service<Request> for BotIdLayer {
    type Error = Infallible;
    type Future = BotIdLayerFuture;
    type Response = Response;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let bot_id = req.bot_id_ref();
        if let Some(bot_id) = bot_id {
            if self.id.eq(bot_id) {
                return BotIdLayerFuture::Future {
                    inner: self.svc.call(req),
                    id:    self.id.clone(),
                };
            }
        }
        BotIdLayerFuture::Ready(Some(Response::none()))
    }
}

#[pin_project(project = BotIdLayerFutureProj)]
pub enum BotIdLayerFuture {
    Future {
        #[pin]
        inner: RouteFuture,
        id:    String,
    },
    Ready(Option<Response>),
}

impl Future for BotIdLayerFuture {
    type Output = Result<Response, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this {
            BotIdLayerFutureProj::Future { inner, id } => {
                let response = ready!(inner.poll(cx));
                Poll::Ready(response.map(|mut r| {
                    r.set_bot_id(id);
                    r
                }))
            }
            BotIdLayerFutureProj::Ready(response) => Poll::Ready(Ok(response.take().expect(
                "If you see this message, it means that the bot ID layer failed to retrieve the \
                 bot ID from the request. THIS IS A BUG. PLEASE REPORT IT.",
            ))),
        }
    }
}
