use std::io;
use deckbuilder::prelude::*;

pub mod util;
use crate::util::*;

pub struct Game {
    players: Table,
    player_amount: u32,
    deck: Deck,
    round: u64,
    end_game: bool
}

impl Game {
    fn init_game_from(loadout: Loadout) -> Self {
        println!("\x1b[1;34m### CLI BLACKJACK ###\x1b[0m");
        println!("Enter the amount of players:");

        let mut response: String = String::new();
        io::stdin()
            .read_line(&mut response)
            .expect("Failed to read line.");

        let player_count: u32 = response.trim().parse::<u32>().expect("Invalid player count");

        let mut p: Vec<Player> = Vec::new();
        for i in 0..player_count {
            p.push(Player(i, Chip::from_loadout(loadout.clone()), Bet(vec![]), true));
        }

        Self { 
            players: Table(p), 
            player_amount: player_count,
            deck: Deck::build(2),
            round: 1,
            end_game: false
        }
    }

    fn init_euro5_game() -> Self {
        Self::init_game_from(Loadout::Euro5)
    }

    fn start_game(&mut self) -> Result<(), DeckError>{
        while !self.end_game {
            let game_over: Option<bool> = self.betting_phase();
            if game_over.is_none() {
                return Err(DeckError::ErrorWhileBetting);
            }

            if game_over.unwrap() {
                println!("\x1b[1;31mAll players have gone bankrupt!\x1b[0m");
                println!();
                println!("\x1b[1;34mHands played:\x1b[0m \t{}", self.round - 1);  
                println!();
    
                for (index, player) in self.players.0.iter().enumerate() {
                    show_final_results(index, player);
                }
                
                break;
            }

            self.game_round()?;
        }

        Ok(())
    }

    fn betting_phase(&mut self) -> Option<bool> {

        if game_over(&self.players.0) {
            return Some(true);
        }

        for player in &mut self.players.0 {
            player.2 = Bet(vec![]);
        }

        println!();
        println!("\x1b[1;34m### Betting Phase ###\x1b[0m");
        
        for player in 0..self.player_amount {

            let player_instance: &mut Player = &mut self.players.0[player as usize];

            if !player_instance.is_active() {
                continue;
            }

            println!("\x1b[1;34mPlayer {}:\x1b[0m\t\x1b[31mBalance: {}\x1b[0m\tPlace your bet:", player + 1, 
                self.players.get_player_by_id(player)?.get_balance());
            
            let mut current_bet: Bet = Bet(vec![]);

            loop {
                println!("'1', '5', '10', '25', '100', '500', '1000', 'All-In', 'Ok'");
                println!("Current bet: \x1b[1;32m{}\x1b[0m", current_bet.sum());
    
                let mut response: String = String::new();
    
                io::stdin()
                    .read_line(&mut response)
                    .expect("Failed to read line");

                match response.as_str().trim() {
                    "1" => current_bet.0.push(Chip::C1),
                    "5" => current_bet.0.push(Chip::C5),
                    "10" => current_bet.0.push(Chip::C10),
                    "25" => current_bet.0.push(Chip::C25),
                    "100" => current_bet.0.push(Chip::C100),
                    "500" => current_bet.0.push(Chip::C500),
                    "1000" => current_bet.0.push(Chip::C1000),
                    "All-In" | "A" | "a" => {
                        let player_balance: u32 = self.players.get_player_by_id(player)?.get_balance();
                        current_bet.0 = player_balance.into_chips();
                        break;
                    },
                    "Ok" => {
                        if current_bet.sum() == 0 {
                            println!("You must place a bet!");
                            continue;
                        } else {
                            break;
                        }
                    },
                    _ => {
                        println!("You must place a bet!");
                        continue;
                    },
                }

                if current_bet.sum() > self.players.get_player_by_id(player)?.get_balance() {
                    println!("You can't bet more than you have!");
                    current_bet.0 = self.players.get_player_by_id(player)?.get_balance().into_chips();
                } 
                else if current_bet.sum() == self.players.get_player_by_id(player)?.get_balance() {
                    break;
                }
            }

            self.players.0[player as usize].2 = current_bet;
        }

        println!();
        println!("\x1b[1;34m### Betting Phase is Over! ###\x1b[0m");
        println!("All bet's were placed!");
        
        for player in 0..self.player_amount {
            println!("\x1b[1;34mPlayer {}:\x1b[0m\tBet: \x1b[31m{}\x1b[0m", player + 1, self.players.0[player as usize].2.sum());
        }

        Some(false)
    }

    fn game_round(&mut self) -> Result<(), DeckError> {
        println!();
        println!("\x1b[1;34m### Round {}! ###\x1b[0m", self.round);
        print!("\n");

        self.deck.reshuffle(2);
        let mut player_hands: Vec<Hand> = Vec::new();

        let mut dealer_hand: Hand = Hand(vec![], false);
        dealer_hand.draw_from(&mut self.deck)?;
        dealer_hand.draw_from_hidden(&mut self.deck)?;

        show_dealer_hand(&dealer_hand);

        for _ in 0..self.player_amount as usize {
            player_hands.push(self.deck.deal_hand(2)?);
        }

        show_player_hands(&self.players, &player_hands, 0);

        for player in 0..self.player_amount as usize {

            if !self.players.0[player].is_active() {
                continue;
            }

            let mut moves: u32 = 0;
            let mut player_stands: bool = false;
            let mut player_busted: bool = false;
            let mut hand: Hand = player_hands[player].clone();

            while !player_stands {
                moves += 1;
                println!("\x1b[1;34mHit: 'H', Double-Down: 'D', Stand: 'S'\x1b[0m");

                let mut response: String = String::new();

                io::stdin()
                    .read_line(&mut response)
                    .expect("Failed to read line");

                match response.as_str().trim() {
                    "H" | "h" => {
                        hand.draw_from(&mut self.deck)?;
                        player_stands = hand.check(player as u32, player_hands.clone());
                        player_busted = player_stands;
                    },
                    "D" | "d" => {
                        if !(moves > 1) {
                            hand.draw_from(&mut self.deck)?;
                            player_busted = hand.check(player as u32, player_hands.clone());
                            player_stands = true;
                        } else {
                            println!("You can't double down on this hand.");
                        }
                    },
                    "S" | "s" => player_stands = true,
                    _ => println!("Invalid response! Please try again."),
                }

                match player_busted {
                    false => player_hands[player] = hand.clone(),
                    true => player_hands[player] = Hand(hand.0.clone(), true),
                }

                match !player_stands {
                    true => show_player_hands(&self.players, &player_hands, player as u32),
                    false => show_player_hands(&self.players, &player_hands, player as u32 + 1),
                };
            }
        }

        let local: Hand = dealer_hand.clone();
        dealer_hand[local.last_index() as usize] = dealer_hand.last()?.reveal();
        show_dealer_hand(&dealer_hand);

        // TODO: Implement dealer logic
        println!("\x1b[1;31mDealer plays...\x1b[0m");
        
        let mut new_hand: Hand = Hand(vec![], false);
        match dealer_logic(&mut self.deck, &mut dealer_hand, true) {
            Ok(final_hand) => {
                println!("\x1b[1;31mDealer's final hand:\x1b[0m\t{}", final_hand);
                new_hand = final_hand.clone();
                if final_hand.busted() {
                    println!("\x1b[1;31mDealer busted!\x1b[0m");
                } else {
                    println!("\x1b[1;31mDealer stands with a total of {}\x1b[0m", final_hand.sum());
                }
            },
            Err(e) => println!("Error during dealer play: {:?}", e), 
        }

        dealer_hand = new_hand;

        println!("\x1b[1;34m### Results of Round {} ###\x1b[0m", self.round);
        println!();

        show_results(&mut self.players, &player_hands, &dealer_hand)?;

        self.round += 1;
        
        let mut response: String = String::new();
    
        println!("\x1b[1;31mNext round: 'Y', Quit: 'Q'\x1b[0m");
    
        io::stdin()
            .read_line(&mut response)
            .expect("Failed to read line!");
    
        if response.trim() == "Q" || response.trim() == "q" {
            self.end_game = true;

            println!();
            println!("\x1b[1;34m### Final Results ###\x1b[0m");
            println!();

            println!("\x1b[1;34mHands played:\x1b[0m \t{}", self.round - 1);  
            println!();

            for (index, player) in self.players.0.iter().enumerate() {
                show_final_results(index, player);
            }
        }

        for (index, player) in self.players.0.clone().iter().enumerate() {
            if player.1.sum() <= 0 && player.is_active() {
                println!("\x1b[1;31mPlayer {} has gone bankrupt!\x1b[0m", index + 1);
                self.players.0[index].bankrupt();
            }
        }

        player_hands.clear();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game: Game = Game::init_euro5_game();
    game.start_game()?;

    Ok(())
}