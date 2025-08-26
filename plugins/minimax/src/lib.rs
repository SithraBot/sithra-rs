use sithra_kit::{
    server::traits::TypedRequest,
    transport::{EncodeError, datapack::RequestDataPack},
};

pub struct GenVoice(pub String);

impl TryFrom<GenVoice> for RequestDataPack {
    type Error = EncodeError;

    fn try_from(value: GenVoice) -> Result<Self, Self::Error> {
        Self::default().path("/minimax/voice_clone_api").payload(&value.0)
    }
}

impl TypedRequest for GenVoice {
    type Response = String;
}
