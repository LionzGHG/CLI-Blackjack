use std::cmp::Ordering;
use deckbuilder::prelude::*;

#[derive(Clone)]
pub struct Player(pub u32, pub Balance, pub Bet, pub bool);

impl Player {
    pub fn get_balance(&self) -> u32 {
        self.1.sum()
    }

    pub fn is_active(&self) -> bool {
        self.3
    }

    pub fn bankrupt(&mut self) {
        self.3 = false
    }
}

pub fn game_over(players: &Vec<Player>) -> bool {
    let total_players: u32 = players.len() as u32;
    let mut total_players_bankrupt: u32 = 0;

    for player in players {
        if !player.is_active() {
            total_players_bankrupt += 1;
        }
    }

    total_players_bankrupt == total_players
}

pub struct Table(pub Vec<Player>);

impl Table {
    pub fn get_player_by_id(&self, id: u32) -> Option<Player> {
        self.0.iter().find(|player: &&Player| player.0 == id).cloned()
    }

    pub fn get_mut_player_by_id(&mut self, id: u32) -> Option<&mut Player> {
        self.0.iter_mut().find(|player: &&mut Player| player.0 == id)
    }
}

pub fn show_results(players: &mut Table, player_hands: &Vec<Hand>, dealer_hand: &Hand) -> Result<(), DeckError> {
    let dealer_busted: bool = dealer_hand.busted();
    
    for (index, player_hand) in player_hands.iter().enumerate() {
        
        if !players.0[index].is_active() {
            continue;
        }

        if let Some(player_instance) = players.get_mut_player_by_id(index as u32) {
            
            if player_hand.busted() {
                println!("\x1b[1;34mPlayer {}:\x1b[0m\tBusted!\t\x1b[1;31m-{}\x1b[0m", 
                    index + 1, 
                    player_instance.2.sum()
                );
                player_instance.1.0 = (player_instance.1.sum() - player_instance.2.sum()).into_chips();
                continue;
            }

            if dealer_busted {
                println!(
                    "\x1b[1;34mPlayer {}:\x1b[0m\tWin! (Dealer Busted)\t\x1b[1;32m{}\x1b[0m",
                    index + 1,
                    player_instance.2.sum() * 2
                );
                player_instance.1.0 = (player_instance.1.sum() + player_instance.2.sum() * 2).into_chips();
                continue;
            }

            if player_hand.is_blackjack() {
                println!("\x1b[1;34mPlayer {}:\x1b[0m\tBlackjack!\t\x1b[1;32m{}\x1b[0m", 
                    index + 1,
                    player_instance.2.sum() * 2
                );
            }

            else {
                match dealer_hand.compare_to(&player_hand)? {
                    Ordering::Equal => {
                        println!("\x1b[1;34mPlayer {}:\x1b[0m\tPush!\t\x1b[1;32m{}\x1b[0m", index + 1, player_instance.2.sum());
                    },
                    Ordering::Less => {
                        println!("\x1b[1;34mPlayer {}:\x1b[0m\tWin!\t\x1b[1;32m{}\x1b[0m", index + 1, player_instance.2.sum() * 2);
                        player_instance.1.0 = (player_instance.1.sum() + player_instance.2.sum() * 2).into_chips();     
                    },
                    Ordering::Greater => {
                        println!("\x1b[1;34mPlayer {}:\x1b[0m\tLoss!\t\x1b[1;31m-{}\x1b[0m", index + 1, player_instance.2.sum());
                        player_instance.1.0 = (player_instance.1.sum() - player_instance.2.sum()).into_chips();
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn show_final_results(index: usize, player: &Player) {
    println!(
        "\x1b[1;34mPlayer {}:\x1b[1;31m\tBalance:\x1b[0m {}\t{}", 
            index + 1, 
            match player.1.sum().cmp(&0) {
                Ordering::Less | Ordering::Equal => String::from("\x1b[1;31mBankrupt!\x1b[0m"),
                Ordering::Greater => format!("\x1b[1;32m{}\x1b[0m", player.1.sum()),
            }, 
            match (player.1.sum() as i32 - 500).cmp(&0) {
                Ordering::Less => format!("\x1b[1;31mLoss: {}\x1b[0m", player.1.sum() as i32 - 500),
                Ordering::Equal | Ordering::Greater => format!("\x1b[1;32m\tWon: {}\x1b[0m", player.1.sum() as i32 - 500),
            }
    );
}

pub fn show_player_hands(table: &Table, player_hands: &Vec<Hand>, active_player: u32) {
    for (player, hand) in player_hands.iter().enumerate() {
        if !table.0[player].is_active() {
            continue;
        }
        if player == active_player as usize {
            println!("\x1b[1;34mplayer {}: {}\x1b[0m", player + 1, hand);
        } else {
            println!("player {}: {}", player + 1, hand);
        }
    }
}

pub fn show_dealer_hand(dealer_hand: &Hand) {
    print!("\n");
    println!("Dealer Cards:");
    println!("{} {}", dealer_hand[0], dealer_hand[1]);
    print!("\n");
}

pub fn busting_probability(deck: &Deck, hand: &Hand) -> Result<f64, DeckError> {
        let hand_value: u32 = hand.level_off_ace();

        if hand_value > 21 {
            return Ok(1.0);
        }

        let mut busting_cards: i32 = 0;
        let total_cads: f64 = deck.total_cards() as f64;

        for card in deck.cards() {
            let potential_sum: u32 = hand_value + card.value();
            let adjusted_sum: u32 = if hand.contains(Rank::Ace) && potential_sum > 21 {
                potential_sum - 10
            } else {
                potential_sum
            };

            if adjusted_sum > 21 {
                busting_cards += 1;
            }
        }

        Ok(busting_cards as f64 / total_cads)
}

pub fn dealer_logic<'a>(deck: &'a mut Deck, dealer_hand: &'a mut Hand, hit_on_soft_17: bool) -> Result<&'a mut Hand, DeckError> {
    loop {
        let total: u32 = dealer_hand.sum();
        let is_soft: bool = dealer_hand.contains(Rank::Ace) && total <= 21 && total - 10 > 0 && total - 10 <= 21;

        if total > 21 {
            dealer_hand.1 = true;
            break;
        }

        if total > 17 || (total == 17 && (!hit_on_soft_17 || is_soft)) {
            break;
        }

        dealer_hand.draw_from(deck)?;
    }

    Ok(dealer_hand)
}