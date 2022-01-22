use async_trait::async_trait;
use json_rpc2::{futures::*, Request, Response, Result};
use serde_json::Value;
use tokio::try_join;

struct ServiceHandler;

impl ServiceHandler {
    pub fn new() -> Box<dyn Service<Data = ()>> {
        return Box::new(ServiceHandler {});
    }
}

#[async_trait]
impl Service for ServiceHandler {
    type Data = ();

    async fn handle(
        &self,
        request: &Request,
        _ctx: &Self::Data,
    ) -> Result<Option<Response>> {
        let response = match request.method() {
            "hello" => {
                let params: String = request.deserialize()?;
                let message = format!("Hello, {}!", params);
                Some((request, Value::String(message)).into())
            }
            _ => None,
        };
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = ServiceHandler::new();
    let server = Server::new(vec![service]);

    try_join!(
        tokio::spawn(async move {
            let request = Request::new_reply("hello", Some(Value::String("world".to_string())));
            let response = server.serve(&request, &()).await;

            println!("{:?}", response.as_ref().unwrap().result());

            assert_eq!(
                Some(Value::String("Hello, world!".to_string())),
                response.unwrap().into()
            );
        }),
        tokio::spawn(async move {}),
    ).unwrap();

    Ok(())
}
