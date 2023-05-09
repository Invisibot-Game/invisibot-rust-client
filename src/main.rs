use std::net::TcpStream;

use invisibot_game::{
    clients::{game_message::GameMessage, round_response::RoundResponse},
    utils::direction::Direction,
};
use rand::{thread_rng, Rng};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};
type WS = WebSocket<MaybeTlsStream<TcpStream>>;

mod bot;

fn main() {
    let (mut conn, _) =
        tungstenite::connect("ws://localhost:4900").expect("Failed to connect to server");

    listen_on_server(&mut conn);

    conn.close(None).unwrap();
}

fn listen_on_server(conn: &mut WS) {
    let mut rng = thread_rng();
    let mut turns_until_shoot = rng.gen_range(3..=7);
    let mut prev_move: RoundResponse = RoundResponse::Shoot;

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
                let round_move = if turns_until_shoot == 0 {
                    turns_until_shoot = rng.gen_range(3..=7);
                    RoundResponse::Shoot
                } else {
                    turns_until_shoot -= 1;
                    bot::handle_round(&game_round, &prev_move)
                };
                prev_move = round_move.clone();
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
