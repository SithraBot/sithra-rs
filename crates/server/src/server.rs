//! Provides a `Server` and `Client` for handling `sithra` transport layer
//! connections.
//!
//! The `Server` is responsible for processing incoming requests and sending
//! responses. It uses a `tower::Service` to handle application logic.
//!
//! The `Client` provides a simple way to send requests to the `Server` and
//! receive responses.

use std::convert::Infallible;

use either::Either;
use futures_util::{FutureExt, SinkExt, StreamExt, future::Map};
use sithra_transport::{
    EncodeError,
    datapack::{DataPack, DataPackCodec, DataPackCodecError, RequestDataPack},
    peer::{Reader, Writer},
};
use thiserror::Error;
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender, error::SendError},
        oneshot,
    },
    task::JoinSet,
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tower::Service;
use ulid::Ulid;

use crate::{
    request::Request,
    response::Response,
    shared::{ReceiverGuard, SharedOneshotMap},
    traits::TypedRequest,
};

/// The core server component for handling connections.
///
/// A `Server` is created using `Server::new()` and configured with a
/// `tower::Service`. The service is responsible for handling incoming requests
/// and returning responses. The `serve` method starts the server and handles
/// the communication with a client over a provided `Reader` and `Writer`.
///
/// The generic type `S` represents the `tower::Service` that will handle
/// requests. When first created with `Server::new()`, `S` is `()`. The
/// `service` method is used to replace the unit type with a concrete service
/// implementation.
pub struct Server<S = ()> {
    service:            S,
    writer_rx:          UnboundedReceiver<DataPack>,
    writer_tx:          UnboundedSender<DataPack>,
    request_rx:         UnboundedReceiver<Request>,
    request_tx:         UnboundedSender<Request>,
    response_rx:        UnboundedReceiver<DataPack>,
    response_tx:        UnboundedSender<DataPack>,
    shared_oneshot_map: SharedOneshotMap<Ulid, DataPack>,
}

/// A client for communicating with a `Server`.
///
/// A `Client` is obtained by calling the `client()` method on a `Server`
/// instance. It allows sending `RequestDataPack`s to the server and receiving
/// responses asynchronously.
pub struct Client {
    writer_tx:          UnboundedSender<DataPack>,
    shared_oneshot_map: SharedOneshotMap<Ulid, DataPack>,
}

pub struct ClientSink {
    writer_tx: UnboundedSender<DataPack>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            writer_tx:          self.writer_tx.clone(),
            shared_oneshot_map: self.shared_oneshot_map.clone(),
        }
    }
}

impl Default for Server<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Server<()> {
    /// Creates a new `Server` instance with default settings.
    ///
    /// The initial server is created without a service. The `service` method
    /// must be called to provide a `tower::Service` that will handle requests.
    #[must_use]
    pub fn new() -> Self {
        let (writer_tx, writer_rx) = tokio::sync::mpsc::unbounded_channel();
        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel();
        let (response_tx, response_rx) = tokio::sync::mpsc::unbounded_channel();

        Self {
            service: (),
            writer_rx,
            writer_tx,
            request_rx,
            request_tx,
            response_rx,
            response_tx,
            shared_oneshot_map: SharedOneshotMap::new(),
        }
    }
}

impl<S> Server<S> {
    /// Attaches a `tower::Service` to the server.
    ///
    /// The provided service will be used to process incoming requests.
    ///
    /// # Arguments
    ///
    /// * `svc` - A `tower::Service` that handles `Request`s and returns
    ///   `Response`s.
    ///
    /// # Returns
    ///
    /// A new `Server` instance configured with the provided service.
    #[must_use]
    pub fn service<S1>(self, svc: S1) -> Server<S1>
    where
        S1: Service<Request, Response = Response, Error = Infallible> + Send + 'static,
        S1::Future: Send + 'static,
    {
        let Self {
            service: _,
            writer_rx,
            writer_tx,
            request_rx,
            request_tx,
            response_rx,
            response_tx,
            shared_oneshot_map,
        } = self;
        Server {
            service: svc,
            writer_rx,
            writer_tx,
            request_rx,
            request_tx,
            response_rx,
            response_tx,
            shared_oneshot_map,
        }
    }

    /// Creates a new `Client` connected to this server.
    ///
    /// The returned `Client` can be used to send requests to the server.
    /// Multiple clients can be created, and they can be used from different
    /// threads.
    pub fn client(&self) -> Client {
        Client {
            writer_tx:          self.writer_tx.clone(),
            shared_oneshot_map: self.shared_oneshot_map.clone(),
        }
    }
}

impl<S> Server<S>
where
    S: Service<Request, Response = Response, Error = Infallible> + Send + 'static,
    S::Future: Send + 'static,
{
    /// Starts the server and begins processing requests.
    ///
    /// This method consumes the `Server` and starts four background tasks to
    /// handle:
    /// 1. Receiving responses and completing one-shot channels.
    /// 2. Sending data from the writer channel to the `Writer`.
    /// 3. Reading data from the `Reader` and dispatching it as requests or
    ///    responses.
    /// 4. Processing requests with the `tower::Service` and sending back
    ///    responses.
    ///
    /// # Arguments
    ///
    /// * `writer` - A `Writer` for sending `DataPack`s to the client.
    /// * `reader` - A `Reader` for receiving `DataPack`s from the client.
    ///
    /// # Returns
    ///
    /// A `JoinSet` containing the handles of the background tasks. You can
    /// `await` on the `JoinSet` to wait for the server to stop.
    #[must_use]
    pub fn serve(self, writer: Writer, reader: Reader) -> JoinSet<Result<(), ServerError>> {
        let Self {
            service,
            writer_rx,
            writer_tx,
            request_rx,
            request_tx,
            response_rx,
            response_tx,
            shared_oneshot_map,
        } = self;
        let framed_writer = FramedWrite::new(writer, DataPackCodec::default());
        let framed_reader = FramedRead::new(reader, DataPackCodec::default());
        let mut join_set = JoinSet::new();
        join_set.spawn(async move {
            let mut response_rx = response_rx;
            let shared_oneshot_map = shared_oneshot_map;
            while let Some(response) = response_rx.recv().await {
                let key = response.correlation();
                shared_oneshot_map.complete(&key, response);
            }
            Ok(())
        });
        join_set.spawn(async move {
            let mut writer_rx = writer_rx;
            let mut framed_writer = framed_writer;
            while let Some(data) = writer_rx.recv().await {
                framed_writer.send(data).await?;
            }
            Ok(())
        });
        join_set.spawn(async move {
            let mut framed_reader = framed_reader;
            let request_tx = request_tx;
            let response_tx = response_tx;
            while let Some(data) = framed_reader.next().await {
                let data = data?;
                match data.either_request() {
                    Either::Left(response) => {
                        response_tx.send(response)?;
                    }
                    Either::Right(request_datapack) => {
                        let request = Request::new(request_datapack);
                        request_tx.send(request)?;
                    }
                }
            }
            Ok(())
        });
        join_set.spawn(async move {
            let writer_tx = writer_tx;
            let mut request_rx = request_rx;
            let mut service = service;
            while let Some(request) = request_rx.recv().await {
                let response = service.call(request).await?;
                for response_datapack in response.data {
                    writer_tx.send(response_datapack)?;
                }
            }
            Ok(())
        });
        join_set
    }
}

impl Client {
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
    pub fn post<Error>(
        &self,
        datapack: impl TryInto<RequestDataPack, Error = Error>,
    ) -> Result<ReceiverGuard<Ulid, DataPack>, PostError>
    where
        Error: Into<PostError>,
    {
        let datapack = datapack.try_into().map_err(Into::into)?;
        let key = datapack.correlation();
        let guard = self.shared_oneshot_map.register(key).expect("Ulid Conflict");
        self.writer_tx
            .send(datapack.into())
            .map_err(|err| PostError::ChannelClosed(err.0))?;
        Ok(guard)
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
    pub fn post_typed<Error, T: TypedRequest + TryInto<RequestDataPack, Error = Error>>(
        &self,
        datapack: T,
    ) -> Result<
        Map<
            ReceiverGuard<Ulid, DataPack>,
            impl FnOnce(
                Result<DataPack, oneshot::error::RecvError>,
            ) -> Result<<T as TypedRequest>::Response, PostError>,
        >,
        PostError,
    >
    where
        Error: Into<PostError>,
    {
        let result = self.post(datapack);
        result.map(|fut| {
            fut.map(|rs| match rs {
                Err(err) => Err(err.into()),
                Ok(dp) => Ok(dp.payload::<T::Response>()?),
            })
        })
    }

    /// Sends a request to the server without waiting for a response.
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
    pub fn send<Error>(
        &self,
        datapack: impl TryInto<RequestDataPack, Error = Error>,
    ) -> Result<(), PostError>
    where
        Error: Into<PostError>,
    {
        let datapack = datapack.try_into().map_err(Into::into)?;
        self.writer_tx
            .send(datapack.into())
            .map_err(|err| PostError::ChannelClosed(err.0))?;
        Ok(())
    }

    #[must_use]
    pub fn sink(&self) -> ClientSink {
        ClientSink {
            writer_tx: self.writer_tx.clone(),
        }
    }
}

impl ClientSink {
    /// Sends a request to the server without waiting for a response.
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
    pub fn send<Error>(
        &self,
        datapack: impl TryInto<DataPack, Error = Error>,
    ) -> Result<(), PostError>
    where
        Error: Into<PostError>,
    {
        let datapack = datapack.try_into().map_err(Into::into)?;
        self.writer_tx.send(datapack).map_err(|err| PostError::ChannelClosed(err.0))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum PostError {
    #[error("Channel closed")]
    ChannelClosed(DataPack),
    #[error("Recv error: {0}")]
    RecvError(#[from] oneshot::error::RecvError),
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Encoding error: {0}")]
    EncodingError(#[from] EncodeError),
}

impl From<Infallible> for PostError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

impl From<String> for PostError {
    fn from(value: String) -> Self {
        Self::RequestError(value)
    }
}

/// Represents errors that can occur within the server.
#[derive(Debug, Error)]
pub enum ServerError {
    /// An error occurred during data encoding or decoding.
    #[error("Codec error: {0}")]
    Codec(#[from] DataPackCodecError),
    /// The reader was closed, indicating the connection was lost.
    #[error("Reader closed")]
    ReaderClosed,
    /// An error occurred when sending data through an internal channel.
    /// This typically means a receiver has been dropped.
    #[error("Internal channel Send error")]
    SendError,
    /// An error occurred while waiting for a response on a one-shot channel.
    #[error("Oneshot receive error")]
    OneshotRecvError(#[from] tokio::sync::oneshot::error::RecvError),
    /// An error occurred during deserialization.
    #[error("Deserialize error: {0}")]
    DeserializeError(String),
}

impl<T> From<SendError<T>> for ServerError {
    fn from(_value: SendError<T>) -> Self {
        Self::SendError
    }
}

impl From<Infallible> for ServerError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}
