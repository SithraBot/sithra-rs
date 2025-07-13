use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use futures_util::{FutureExt, future::Map};
use serde::Deserialize;
use sithra_transport::datapack::{DataPack, RequestDataPack};
use tokio::sync::oneshot;
use triomphe::Arc;
use ulid::Ulid;

use crate::{
    extract::FromRequest,
    request::Request,
    response::Error,
    server::{Client, PostError},
    shared::ReceiverGuard,
    traits::{FromRef, TypedRequest},
};

pub struct Context<T: for<'de> Deserialize<'de>, S> {
    pub state:         S,
    pub request:       Request,
    pub payload_cache: T,
    _marker:           PhantomData<T>,
}

impl<T, S> Context<T, S>
where
    T: for<'de> Deserialize<'de>,
{
    /// # Errors
    ///
    /// Returns an error if the payload cannot be deserialized.
    pub const fn payload(&self) -> &T {
        &self.payload_cache
    }
}

impl<T, S> Deref for Context<T, S>
where
    T: for<'de> Deserialize<'de>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.payload_cache
    }
}

impl<T, S> DerefMut for Context<T, S>
where
    T: for<'de> Deserialize<'de>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.payload_cache
    }
}

impl<OuterState, InnerState, T> FromRequest<OuterState> for Context<T, InnerState>
where
    InnerState: FromRef<OuterState>,
    OuterState: Send + Sync,
    T: for<'de> Deserialize<'de>,
{
    type Rejection = Error<rmpv::ext::Error>;

    async fn from_request(
        parts: Arc<RequestDataPack>,
        state: &OuterState,
    ) -> Result<Self, Self::Rejection> {
        let request = Request::from(parts);
        let payload_cache = request.payload()?;
        Ok(Self {
            state: InnerState::from_ref(state),
            request,
            payload_cache,
            _marker: PhantomData,
        })
    }
}

pub trait Clientful {
    fn client(&self) -> &Client;
}

impl Clientful for Client {
    fn client(&self) -> &Client {
        self
    }
}

impl<T, S> Clientful for Context<T, S>
where
    T: for<'de> Deserialize<'de>,
    S: Clientful,
{
    fn client(&self) -> &Client {
        self.state.client()
    }
}

impl<T, S> Context<T, S>
where
    T: for<'de> Deserialize<'de>,
    S: Clientful,
{
    /// Sends a request to the server and returns a future for the response.
    ///
    /// This method sends a `RequestDataPack` to the server and returns a
    /// `ReceiverGuard`. The `ReceiverGuard` is a future that resolves to
    /// the `DataPack` response from the server.
    ///
    /// # Arguments
    ///
    /// * `datapack` - The request data to send. This can be any type that
    ///   converts into a `RequestDataPack`.
    ///
    /// # Errors
    ///
    /// Returns an `Err(DataPack)` if the connection to the server is closed
    /// before the request can be sent. The `DataPack` inside the `Err` is
    /// the original request that failed to be sent.
    ///
    /// # Panics
    ///
    /// This method panics if there is a `Ulid` conflict for the request's
    /// correlation ID. This is extremely unlikely to happen in practice.
    #[allow(clippy::result_large_err)]
    pub fn post<TR: TypedRequest + Into<RequestDataPack>>(
        &self,
        datapack: TR,
    ) -> Result<
        Map<
            ReceiverGuard<Ulid, DataPack>,
            impl FnOnce(
                Result<DataPack, oneshot::error::RecvError>,
            ) -> Result<<TR as TypedRequest>::Response, PostError>,
        >,
        PostError,
    > {
        let datapack: RequestDataPack = datapack.into();
        let datapack = datapack.link(self.request.raw());
        let result = self.state.client().post(datapack);
        result.map(|fut| {
            fut.map(|rs| match rs {
                Err(err) => Err(err.into()),
                Ok(dp) => Ok(dp.payload::<TR::Response>()?),
            })
        })
    }

    /// Sends a request to the server and returns a future for the response.
    ///
    /// This method sends a `RequestDataPack` to the server and returns a
    /// `ReceiverGuard`. The `ReceiverGuard` is a future that resolves to
    /// the `DataPack` response from the server.
    ///
    /// # Arguments
    ///
    /// * `datapack` - The request data to send. This can be any type that
    ///   converts into a `RequestDataPack`.
    ///
    /// # Errors
    ///
    /// Returns an `Err(DataPack)` if the connection to the server is closed
    /// before the request can be sent. The `DataPack` inside the `Err` is
    /// the original request that failed to be sent.
    ///
    /// # Panics
    ///
    /// This method panics if there is a `Ulid` conflict for the request's
    /// correlation ID. This is extremely unlikely to happen in practice.
    #[allow(clippy::result_large_err)]
    pub fn post_raw(
        &self,
        datapack: impl Into<RequestDataPack>,
    ) -> Result<ReceiverGuard<Ulid, DataPack>, PostError> {
        let datapack: RequestDataPack = datapack.into();
        let datapack = datapack.link(self.request.raw());
        self.state.client().post(datapack)
    }
}
