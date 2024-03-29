use super::{Bot, PlayerResponse};
use crate::models::{PieceBag, Plateau, Player};
use serde_json::json;

/// Number of errors that may occure in a row before game ends
const ERROR_THRESHOLD: usize = 6;
/// Time in seconds that a player will be granted before timing out
const DEFAULT_TIMEOUT: usize = 2;

pub struct Engine<'a> {
    players: Vec<Bot<'a>>,
    plateau: Plateau,
    piece_bag: PieceBag,
    move_count: usize,
    player_count: usize,
    history: Vec<PlayerResponse>,
    on_player_response: Box<dyn OnPlayerResponse>,
}

pub struct EngineBuilder<'a> {
    players: Vec<&'a str>,
    plateau: Option<Plateau>,
    piece_bag: Option<PieceBag>,
    on_player_response: Option<Box<dyn OnPlayerResponse>>,
}

impl<'a> EngineBuilder<'a> {
    pub fn with_player2(&mut self, player_path: &'a str) -> &Self {
        self.players.push(player_path);
        self
    }

    pub fn with_plateau(&mut self, plateau: Plateau) -> &Self {
        self.plateau = Some(plateau);
        self
    }

    pub fn with_piecebag(&mut self, piece_bag: PieceBag) -> &Self {
        self.piece_bag = Some(piece_bag);
        self
    }

    pub fn verbose(&mut self) -> &Self {
        self.on_player_response
            .replace(Box::new(PrintOnPlayerResponse {}));
        self
    }

    pub fn finish(&mut self) -> Engine {
        let mut players =
            vec![Bot::new(self.players[0], DEFAULT_TIMEOUT, Player::Player1).unwrap()];

        if let Some(player_path) = self.players.get(1) {
            let player2 = Bot::new(*player_path, DEFAULT_TIMEOUT, Player::Player2).unwrap();
            players.push(player2);
        }

        let plateau = match self.plateau.take() {
            Some(set_plateau) => set_plateau,
            None => Plateau::default(),
        };

        let piece_bag = match self.piece_bag.take() {
            Some(set_piece_bag) => set_piece_bag,
            None => PieceBag::default(),
        };

        let on_player_response = self
            .on_player_response
            .take()
            .unwrap_or(Box::new(DefaultOnPlayerResponse {}));

        Engine {
            player_count: players.len(),
            players,
            plateau,
            piece_bag,
            move_count: 0,
            history: vec![],
            on_player_response,
        }
    }
}

impl<'a> Engine<'a> {
    pub fn builder<'b>(player_path: &'b str) -> EngineBuilder {
        EngineBuilder {
            players: vec![player_path],
            plateau: None,
            piece_bag: None,
            on_player_response: None,
        }
    }

    pub fn run(&mut self) {
        let mut errors: usize = 0;

        for bot in self.players.iter() {
            println!("Player {}: {}", bot.player(), bot.name())
        }

        loop {
            let response = self.next_move();
            &self.on_player_response.on_player_move(self, &response);

            match &response.error {
                None => errors = 0,
                Some(_) if errors >= ERROR_THRESHOLD => break,
                Some(_) => errors += 1,
            }
            self.history.push(response);
        }

        let placements = self.placement_counts();
        println!("Final Score: ");
        for (player, count) in placements {
            println!("<{}> -> {}", player, count);
        }
    }

    pub fn next_move(&mut self) -> PlayerResponse {
        let player_com = &mut self.players[self.move_count % self.player_count];
        self.move_count += 1;

        let piece = self.piece_bag.next();
        player_com.request_placement(&mut self.plateau, &piece)
    }

    pub fn plateau(&self) -> &Plateau {
        &self.plateau
    }

    pub fn placement_counts(&self) -> Vec<(Player, usize)> {
        self.players
            .iter()
            .map(|player_com| (player_com.player(), player_com.placement_count()))
            .collect()
    }

    pub fn player_names(&self) -> Vec<String> {
        self.players.iter().map(|bot| bot.name()).collect()
    }

    pub fn replay(&self) -> String {
        json!({
        "players": &self.player_names(),
        "plateau": json!({
            "width": self.plateau.width(),
            "height": self.plateau.height(),
            "player1_start": self.plateau.player_start(Player::Player1),
            "player2_start": self.plateau.player_start(Player::Player2),
        }),
        "history": self.history
        })
        .to_string()
    }
}

trait OnPlayerResponse {
    fn on_player_move(&self, engine: &Engine, player_response: &PlayerResponse);
}

struct DefaultOnPlayerResponse;

impl OnPlayerResponse for DefaultOnPlayerResponse {
    fn on_player_move(&self, _: &Engine, _: &PlayerResponse) {}
}

struct PrintOnPlayerResponse;

impl OnPlayerResponse for PrintOnPlayerResponse {
    fn on_player_move(&self, engine: &Engine, player_response: &PlayerResponse) {
        match &player_response.error {
            None => {
                print!(
                    "<got ({}): {}",
                    player_response.player,
                    player_response.raw_response.as_ref().unwrap()
                );
                print!("{}", player_response.piece);
                print!("{}", engine.plateau());
                ()
            }
            Some(e) => {
                println!("{}: {}", player_response.player, e);
            }
        }
    }
}
