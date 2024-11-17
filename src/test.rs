use crate::prelude::*;

#[test]
fn build_deck_test() {
    let knack_deck: Deck = Deck::build(1);
    let poker_deck: Deck = Deck::build(2);

    println!("{knack_deck}");
    print!("\n\n");
    println!("{poker_deck}");
}

#[test]
fn shuffle_deck_test() {
    let mut deck: Deck = Deck::build(1);
    deck.shuffle();
    println!("{deck}");
}

#[test]
fn test_hiding() {
    let mut card: Card = Card { suit: Suit::Clubs, rank: Rank::Ace, hidden: false };
    card.hide();
    println!("hidden: {card}");
    card.reveal();
    println!("revealed: {card}");
}