pub mod botid;
pub mod channel;
pub mod client;
pub mod context;
pub mod correlation;
pub mod from_ref;
pub mod payload;
pub mod state;

use std::convert::Infallible;

use sithra_transport::datapack::RequestDataPack;
use triomphe::Arc;

use crate::response::IntoResponse;

pub trait FromRequest<S>: Sized {
    /// If the extractor fails it'll use this "rejection" type. A rejection is
    /// a kind of error that can be converted into a response.
    type Rejection: IntoResponse;

    /// Perform the extraction.
    fn from_request(
        req: Arc<RequestDataPack>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}

impl<S: Sync> FromRequest<S> for () {
    type Rejection = Infallible;

    async fn from_request(_req: Arc<RequestDataPack>, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(())
    }
}
