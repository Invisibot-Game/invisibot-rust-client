use std::{env, net::TcpStream, str::FromStr};

use invisibot_client_api::{
    connect_response::{ClientType, ConnectResponse},
    game_message::GameMessage,
    round_response::RoundResponse,
};
use invisibot_common::GameId;
use rand::{thread_rng, Rng};
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

type WS = WebSocket<MaybeTlsStream<TcpStream>>;

mod bot;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!(
            "Invalid, expected exactly 1 argument for game id, got '{}'",
            args.join(", ")
        );
    }

    let id = args.last().unwrap();
    let game_id = GameId::from_str(id).expect("Invalid UUID");

    let (mut conn, _) =
        tungstenite::connect("ws://localhost:4900").expect("Failed to connect to server");

    listen_on_server(&mut conn, game_id);

    conn.close(None).unwrap();
}

fn listen_on_server(conn: &mut WS, game_id: GameId) {
    let mut rng = thread_rng();
    let mut turns_until_shoot = rng.gen_range(3..=7);
    let mut prev_move: RoundResponse = RoundResponse::Shoot;

    loop {
        let msg = conn
            .read_message()
            .expect("Failed to read message from server");

        let message_text = msg.into_text().expect("Message was not 'text'?");
        let Ok(parsed): Result<GameMessage, _> = serde_json::from_str(&message_text) else { continue; };

        println!("==> {}", parsed.message_type());
        match parsed {
            GameMessage::ClientHello => {
                let connect_response = ConnectResponse {
                    game_id,
                    client_type: ClientType::Player,
                };
                let serialized = serde_json::to_string(&connect_response)
                    .expect("Failed to serialize ClientHello response");
                conn.write_message(Message::text(serialized))
                    .expect("Failed to send connect response");
            }
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
            GameMessage::PlayerKilled(id) => {
                println!("Player died {id}");
                return;
            }
            GameMessage::PlayerWon(id) => {
                println!("We won! (Also we had id {id}");
                return;
            }
            GameMessage::GameNotFound(id) => {
                println!("No such game {id}");
                return;
            }
            GameMessage::GameStarted => {
                println!("The game has already started");
                return;
            }
            GameMessage::GameRoundSpectators(_) => { /* Not relevant to us */ }
        }
    }
}
