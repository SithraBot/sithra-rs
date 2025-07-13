use sithra_server::{
    extract::{payload::Payload, state::State},
    on,
    routing::router::Router,
    server::{Client, Server},
    traits::FromRef,
};
use sithra_transport::{datapack::RequestDataPack, peer::Peer};

#[derive(Clone)]
struct AppState {
    client: Client,
}

impl FromRef<AppState> for Client {
    fn from_ref(state: &AppState) -> Self {
        state.client.clone()
    }
}

async fn hello_world(State(client): State<Client>) -> Result<Payload<()>, String> {
    let Ok(response) =
        client.post(RequestDataPack::default().path("/print").payload("hello world!"))
    else {
        return Err("Failed to send request".to_owned());
    };
    let response = response.await.map_err(|_| "Failed to receive response".to_owned())?;
    let response = response.payload::<String>()?;
    assert_eq!(response, "hello world!");
    Ok(Payload(()))
}

#[tokio::main]
async fn main() {
    let peer = Peer::default();
    let (writer, reader) = peer.split();
    let router = Router::new().route("/hello_world", on(hello_world));
    let server = Server::new();
    let client = server.client();
    let router: Router = router.with_state(AppState { client });
    server.service(router).serve(writer, reader).join_all().await;
}
