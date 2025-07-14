pub mod conf;
pub mod loader;

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use sithra_kit::{transport::datapack::DataPack, types::initialize::Initialize};

    #[derive(Deserialize, Serialize)]
    struct A {
        value: String,
    }
    #[test]
    fn init() {
        let init = Initialize::new(
            A {
                value: "hello".to_owned(),
            },
            "test",
            "/data",
        );
        let pack = DataPack::builder().payload(init).path("/").build();
        let raw = pack.serialize_to_raw().unwrap();
        let data = DataPack::deserialize(&raw.data).unwrap();
        let init: Initialize<A> = data.payload().unwrap();
        assert_eq!(init.config.value, "hello");
    }
}
