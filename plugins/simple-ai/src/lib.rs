use sithra_kit::{
    server::traits::TypedRequest,
    transport::{EncodeError, datapack::RequestDataPack},
};

pub struct OnceChat(pub String);

impl TryFrom<OnceChat> for RequestDataPack {
    type Error = EncodeError;

    fn try_from(value: OnceChat) -> Result<Self, Self::Error> {
        Self::default().path("/simple-ai/once_chat_api").payload(&value.0)
    }
}

impl TypedRequest for OnceChat {
    type Response = String;
}
