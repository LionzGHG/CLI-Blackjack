pub mod prelude;
#[cfg(test)]
pub mod test;

use rand::prelude::*;

#[derive(Debug)]
pub enum DeckError {
    EmptyDeck,
    ErrorWhileBetting
}

impl std::error::Error for DeckError {}

impl std::fmt::Display for DeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Clone)]
pub struct Hand(pub Vec<Card>, pub bool);

impl Hand {
    pub fn busted(&self) -> bool {
        self.1
    }
}

impl std::ops::Index<usize> for Hand {
    type Output = Card;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Hand {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Hand {
    pub fn draw_from(&mut self, deck: &mut Deck) -> Result<Card, DeckError> {
        let card: Option<Card> = deck.0.pop();
        match card.is_none() {
            true => return Err(DeckError::EmptyDeck),
            _ => self.0.push(card.unwrap()),
        }
        Ok(card.unwrap())
    }

    pub fn last(&self) -> Result<Card, DeckError> {
        match self.0.last() {
            None => Err(DeckError::EmptyDeck),
            Some(card) => Ok(*card),
        }
    }

    pub fn last_index(&self) -> u32 {
        let mut length: usize = 0;
        let _ = &self.0.iter().for_each(|_| length += 1);
        length as u32 - 1
    }

    pub fn compare_to(&self, other_hand: &Hand) -> Result<std::cmp::Ordering, DeckError> {
        match self.sum().cmp(&other_hand.sum()) {
            std::cmp::Ordering::Equal => Ok(std::cmp::Ordering::Equal),
            std::cmp::Ordering::Greater => Ok(std::cmp::Ordering::Greater),
            std::cmp::Ordering::Less => Ok(std::cmp::Ordering::Less),
        }
    }

    pub fn draw_from_hidden(&mut self, deck: &mut Deck) -> Result<Card, DeckError> {
        let mut card: Card = match deck.0.pop() {
            Some(card) => card,
            None => return Err(DeckError::EmptyDeck),
        };
        card.hide();
        self.0.push(card);
        Ok(card)
    }

    pub fn sum(&self) -> u32 {
        let mut total: u32 = 0;
        for card in &self.0 {
            total += card.rank.get_value();
        }
        total
    }

    pub fn is_blackjack(&self) -> bool {
        self.sum() == 21 && self.0.len() == 2
    }

    pub fn is_bust(&self, sum: u32) -> bool {
        sum > 21
    }

    pub fn contains(&self, rank: Rank) -> bool {
        for card in &self.0 {
            if card.rank == rank {
                return true;
            }
        }
        false
    }

    pub fn level_off_ace(&self) -> u32 {
        if self.is_bust(self.sum()) && self.contains(Rank::Ace) {
            return self.sum() - 10;
        }
        self.sum()
    }

    pub fn check(&self, player_index: u32, mut player_hands: Vec<Hand>) -> bool {
        if self.is_blackjack() {
            return true;
        }
        let total: u32 = self.level_off_ace();
        if self.is_bust(total) {
            println!("\x1b[1;31mplayer {} busted!\x1b[0m", player_index + 1);
            player_hands[player_index as usize] = Hand(Vec::new(), true);
            return true;
        }
        return false;
    }
}

impl std::fmt::Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for card in &self.0 {
            write!(f, "{card}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Deck(Vec<Card>);

impl Deck {
    pub fn build(multiplier: u32) -> Deck {
        let mut deck: Vec<Card> = Vec::<Card>::new();

        for _ in 0..multiplier {
            for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
                for rank in [Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace] {
                    deck.push(Card { suit, rank, hidden: false });
                }
            }
        }

        Deck(deck)
    }

    pub fn shuffle(&mut self) {
        for i in (0..self.0.len()).rev() {
            let j = rand::thread_rng().gen_range(0..i + 1);
            self.0.swap(i, j);
        }
    }

    pub fn reshuffle(&mut self, multiplier: u32) {
        let deck: Deck = Deck::build(multiplier);
        *self = deck;
        self.shuffle();
    }

    pub fn total_cards(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn cards(&self) -> &Vec<Card> {
        &self.0
    }

    pub fn deal_hand(&mut self, cards: u32) -> Result<Hand, DeckError> {
        let mut hand: Vec<Card> = Vec::<Card>::new();

        for _ in 0..cards {
            let card: Option<Card> = self.0.pop();
            match card.is_none() {
                true => return Err(DeckError::EmptyDeck),
                _ => hand.push(card.unwrap()),
            }
        }

        Ok(Hand(hand, false))
    }
}

impl std::fmt::Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for card in &self.0 {
            write!(f, "{} ", card)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Card {
    suit: Suit,
    rank: Rank,
    hidden: bool,
}

impl Card {
    pub fn hide(&mut self) {
        self.hidden = true;
    }

    pub fn reveal(&mut self) -> Card {
        self.hidden = false;
        return *self;
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn value(&self) -> u32 {
        self.rank.get_value()
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.hidden {
            false => write!(f, "{} {}", self.suit, self.rank),
            true => write!(f, "\x1b[1;32m■■\x1b[0m"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Suit {
    Diamonds,
    Hearts,
    Clubs,
    Spades,
}

impl std::fmt::Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diamonds => write!(f, "{}", "\x1b[31m♦\x1b[0m"),
            Self::Hearts => write!(f, "{}", "\x1b[31m♥\x1b[0m"),
            Self::Clubs => write!(f, "{}", "\x1b[36m♣\x1b[0m"),
            Self::Spades => write!(f, "{}", "\x1b[36m♠\x1b[0m"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace,
}

impl Rank {
    pub fn get_value(&self) -> u32 {
        match self {
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten | Self::Jack | Self::Queen | Self::King => 10,
            Self::Ace => 11
        }
    }
}

impl std::fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
            Self::Ace => "A"
        })
    }
}

#[derive(Debug, Clone)]
pub struct Bet(pub Vec<Chip>);

impl Bet {
    pub fn sum(&self) -> u32 {
        self.0.iter().map(|chip: &Chip| chip.value()).sum()
    }
}

#[derive(Debug, Clone)]
pub enum Loadout {
    Euro5,
    CustomLoadout(Vec<Chip>),
}

#[derive(Debug, Clone)]
pub struct Balance(pub Vec<Chip>);

impl Balance {
    pub fn sum(&self) -> u32 {
        self.0.iter().map(|chip: &Chip| chip.value()).sum()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Chip {
    C1,
    C5,
    C10,
    C25,
    C100,
    C500,
    C1000
}

pub trait IntoChips {
    fn into_chips(self) -> Vec<Chip>;
}

impl IntoChips for u32 {
    fn into_chips(self) -> Vec<Chip> {
        let mut value: u32 = self;
        let mut chips: Vec<Chip> = Vec::new();
        
        for chip in Chip::all_chips() {
            while value >= chip.value() {
                chips.push(chip);
                value -= chip.value();
            }
        }
        
        chips
    }
}

impl Chip {
    pub fn from_loadout(loadout: Loadout) -> Balance {
        match loadout {
            Loadout::Euro5 => Balance(vec![
                Chip::C100, Chip::C100, Chip::C100, Chip::C100,
                Chip::C10, Chip::C10, Chip::C10, Chip::C10, Chip::C10, Chip::C10,
                Chip::C5, Chip::C5, Chip::C5, Chip::C5, Chip::C5, Chip::C5, Chip::C5, Chip::C5
            ]),
            Loadout::CustomLoadout(vec) => Balance(vec),
        }
    }

    pub fn all_chips() -> Vec<Chip> {
        vec![
            Chip::C1,
            Chip::C5,
            Chip::C10,
            Chip::C25,
            Chip::C100,
            Chip::C500,
            Chip::C1000
        ]
    }

    pub fn value(&self) -> u32 {
        match self {
            Self::C1 => 1,
            Self::C5 => 5,
            Self::C10 => 10,
            Self::C25 => 25,
            Self::C100 => 100,
            Self::C500 => 500,
            Self::C1000 => 1000,
        }
    }
}