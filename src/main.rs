use std::net::TcpStream;

use invisibot_game::{
    clients::{game_message::GameMessage, round_response::RoundResponse},
    utils::direction::Direction,
};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};
type WS = WebSocket<MaybeTlsStream<TcpStream>>;

mod bot;

fn main() {
    println!("Hello, world!");

    let (mut conn, _) =
        tungstenite::connect("ws://localhost:4900").expect("Failed to connect to server");

    listen_on_server(&mut conn);

    conn.close(None).unwrap();
}

fn listen_on_server(conn: &mut WS) {
    let mut prev_move: Option<Direction> = None;
    loop {
        let msg = conn
            .read_message()
            .expect("Failed to read message from server");

        let message_text = msg.into_text().expect("Message was not 'text'?");
        let parsed: GameMessage =
            serde_json::from_str(&message_text).expect("Failed to parse message");

        println!("==> {}", parsed.message_type());
        match parsed {
            GameMessage::GameRound(game_round) => {
                let dir = bot::handle_round(&game_round, &prev_move);
                prev_move = dir.clone();
                let round_move = RoundResponse::new(dir.unwrap_or(Direction::Down));
                println!("<== {round_move:?}");

                let serialized =
                    serde_json::to_string(&round_move).expect("Failed to serialize round response");
                conn.write_message(Message::text(serialized))
                    .expect("Failed to send round response");
            }
            GameMessage::ClientGoodbye(msg) => {
                println!("Server shutting down with message: {msg}");
                return;
            }
            _ => {}
        }
    }
}
