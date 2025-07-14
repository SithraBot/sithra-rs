use std::{
    convert::Infallible,
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use pin_project::pin_project;
use serde::Serialize;
use sithra_transport::{
    channel::Channel,
    datapack::{DataPack, RequestDataPack},
};
use smallvec::SmallVec;
use tower::Service;
use ulid::Ulid;

use crate::{extract::payload::Payload, request::Request};

pub struct Response {
    pub data: SmallVec<[DataPack; 1]>,
}

pub struct Error<E: Display>(E);

impl<S> From<S> for Error<S>
where
    S: Display,
{
    fn from(value: S) -> Self {
        Self(value)
    }
}

impl Response {
    #[must_use]
    pub fn new(data: impl Into<DataPack>) -> Self {
        Self {
            data: SmallVec::from([data.into()]),
        }
    }

    #[must_use]
    pub fn none() -> Self {
        Self {
            data: SmallVec::new(),
        }
    }

    #[must_use]
    pub fn is_none(&self) -> bool {
        self.data.is_empty()
    }

    pub fn correlate(&mut self, id: Ulid) {
        for data in &mut self.data {
            data.correlate(id);
        }
    }

    pub fn set_bot_id(&mut self, bot_id: impl Display) {
        for data in &mut self.data {
            data.bot_id = Some(bot_id.to_string());
        }
    }

    pub fn set_channel(&mut self, channel: &Channel) {
        for data in &mut self.data {
            data.channel = Some(channel.clone());
        }
    }

    pub fn error(error: impl Display) -> Self {
        Self {
            data: SmallVec::from([DataPack::builder().build_with_error(error)]),
        }
    }
}

pub trait IntoResponse {
    /// Create a response.
    #[must_use]
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl<const N: usize, R: IntoResponse> IntoResponse for [R; N] {
    fn into_response(self) -> Response {
        let mut result = SmallVec::new();
        for response in self {
            result.append(&mut response.into_response().data);
        }
        Response { data: result }
    }
}

impl<const N: usize, R: IntoResponse> IntoResponse for SmallVec<[R; N]> {
    fn into_response(self) -> Response {
        let mut result = SmallVec::new();
        for response in self {
            result.append(&mut response.into_response().data);
        }
        Response { data: result }
    }
}

impl<R: IntoResponse> IntoResponse for Vec<R> {
    fn into_response(self) -> Response {
        let mut result = SmallVec::new();
        for response in self {
            result.append(&mut response.into_response().data);
        }
        Response { data: result }
    }
}

impl IntoResponse for RequestDataPack {
    fn into_response(self) -> Response {
        Response::new(self)
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response {
            data: SmallVec::new(),
        }
    }
}

impl<R: IntoResponse> IntoResponse for Option<R> {
    fn into_response(self) -> Response {
        Response {
            data: self.map_or_else(SmallVec::new, |v| v.into_response().data),
        }
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> Response {
        match self {}
    }
}

impl IntoResponse for DataPack {
    fn into_response(self) -> Response {
        Response {
            data: SmallVec::from([self]),
        }
    }
}

impl<S> IntoResponse for Error<S>
where
    S: Display,
{
    fn into_response(self) -> Response {
        Response::error(self.0)
    }
}

impl<V, E> IntoResponse for Result<V, E>
where
    V: IntoResponse,
    E: Display,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(error) => Response::error(error),
        }
    }
}

impl<V: Serialize> IntoResponse for Payload<V> {
    fn into_response(self) -> Response {
        let Self(payload) = self;
        let value = sithra_transport::to_value(payload);
        let Ok(value) = value else {
            return Response::error("Failed to serialize payload");
        };
        DataPack::builder().build_with_payload(value).into_response()
    }
}

#[derive(Clone)]
pub(crate) struct MapIntoResponse<S> {
    inner: S,
}

impl<S> MapIntoResponse<S> {
    pub(crate) const fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Service<Request> for MapIntoResponse<S>
where
    S: Service<Request>,
    S::Response: IntoResponse,
{
    type Error = S::Error;
    type Future = MapIntoResponseFuture<S::Future>;
    type Response = Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        MapIntoResponseFuture {
            inner: self.inner.call(req),
        }
    }
}

#[pin_project]
pub(crate) struct MapIntoResponseFuture<F> {
    #[pin]
    inner: F,
}

impl<F, T, E> Future for MapIntoResponseFuture<F>
where
    F: Future<Output = Result<T, E>>,
    T: IntoResponse,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = ready!(self.project().inner.poll(cx)?);
        Poll::Ready(Ok(res.into_response()))
    }
}
