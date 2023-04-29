use invisibot_game::{
    clients::{game_message::GameRound, round_response::RoundResponse},
    game_logic::game_map::TileType,
    utils::{coordinate::Coordinate, direction::Direction},
};

pub fn handle_round(game_round: &GameRound, prev_move: &Direction) -> RoundResponse {
    let dir = get_free_directions(game_round)
        .into_iter()
        .filter(|d| d != &prev_move.opposite())
        .last()
        .expect("No free dirs to go to? :(")
        .clone();
    RoundResponse::new(dir)
}

fn get_free_directions(game_round: &GameRound) -> Vec<Direction> {
    let me = game_round
        .visible_players
        .get(&game_round.own_player_id)
        .expect("Own player not in list?");

    Direction::all_dirs()
        .into_iter()
        .filter_map(|dir| get_tile_at(&me.translate(&dir), game_round).map(|_| dir))
        .collect()
}

fn get_tile_at(coord: &Coordinate, game_round: &GameRound) -> Option<TileType> {
    if coord.x >= game_round.width || coord.y >= game_round.height {
        return None; // Outside of bounds
    }

    if game_round.walls.contains(coord) {
        return None;
    }

    if game_round
        .visible_players
        .iter()
        .filter(|&(id, _)| id != &game_round.own_player_id)
        .filter(|&(_, p)| p == coord)
        .count()
        > 0
    {
        return None; // A player is occuping the tile
    }

    return Some(TileType::Empty);
}