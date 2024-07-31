use actix::prelude::*;
use actix::{Actor, Addr, Running, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::sync::{Arc, Mutex};
pub type P2PClientsType = Arc<Mutex<Vec<Addr<P2PWebSocket>>>>;

#[derive(Serialize, Deserialize, Debug)]
pub enum P2PMessage {
    query_latest,
    query_all,
    response_blockchain(String),
}

pub struct P2PWebSocket {
    clients: P2PClientsType,
}

// Define a wrapper type that implements actix::Message
#[derive(Message)]
#[rtype(result = "()")]
struct BroadcastMessage(String);
impl Handler<BroadcastMessage> for P2PWebSocket {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl P2PWebSocket {
    pub fn new(clients: P2PClientsType) -> Self {
        Self { clients }
    }

    pub fn broadcast(clients: &Vec<Addr<P2PWebSocket>>, message: P2PMessage) -> Result<()> {
        let serialized_message = serde_json::to_string(&message)?;
        for client in clients.iter() {
            client.do_send(BroadcastMessage(serialized_message.clone()));
        }
        Ok(())
    }
}

impl Actor for P2PWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("connection started ");
        self.clients.lock().unwrap().push(ctx.address());
    }
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        println!("stopping...");
        let mut clients = self.clients.lock().unwrap();
        if let Some(pos) = clients.iter().position(|addr| addr == &ctx.address()) {
            clients.remove(pos);
        }
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for P2PWebSocket {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("msg {:?}", item);
        match item {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<P2PMessage>(&text) {
                    Ok(message) => {
                        match message {
                            P2PMessage::query_all => ctx.text("query all"),
                            P2PMessage::query_latest => ctx.text("query latest"),
                            P2PMessage::response_blockchain(t) => ctx.text(t),
                        }
                    },
                    Err(e) => {
                        println!("Failed to deserialize message: {:?}", e);
                        ctx.stop();
                    },
                }
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                println!("ctx close");
                ctx.close(reason);
                ctx.stop();
            }
            _ => {
                println!("ctx stop");
                ctx.stop()
            }
        }
    }
}
