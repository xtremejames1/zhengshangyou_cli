use crate::card;
use crate::hand;
use crate::play;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::{
    cursor, execute, queue,
    style::{self, Color, Stylize},
    terminal,
};
use std::io::{self, Write};
use std::time::Duration;

pub fn init() {
    terminal::enable_raw_mode();
    queue!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        style::SetBackgroundColor(style::Color::DarkGreen),
        cursor::Hide
    );
    io::stdout().flush();
}

pub fn get_keystate() -> Input_States {
    if poll(Duration::from_millis(500)).ok().expect("IDK") {
        let event = read();
        match event {
            Ok(Event::Key(event)) if event.kind == KeyEventKind::Press => match event.code {
                KeyCode::Esc => {
                    return Input_States::Esc;
                }
                KeyCode::Left => {
                    return Input_States::Left;
                }
                KeyCode::Right => {
                    return Input_States::Right;
                }
                KeyCode::Enter => {
                    return Input_States::Enter;
                }
                KeyCode::Char(' ') => {
                    return Input_States::Space;
                }

                _ => {
                    return Input_States::Empty;
                }
            },
            _ => {
                return Input_States::Empty;
            }
        }
    }
    Input_States::Empty
}

pub fn warn(a: String) {
    queue!(
        io::stdout(),
        cursor::MoveTo(
            (terminal::size().unwrap().0 - a.len() as u16) / 2,
            terminal::size().unwrap().1 * 3 / 4
        ),
        style::SetBackgroundColor(style::Color::DarkRed),
        style::PrintStyledContent(a.white())
    );
    io::stdout().flush();
}

pub fn announce(a: String) {
    queue!(
        io::stdout(),
        cursor::MoveTo(
            (terminal::size().unwrap().0 - a.len() as u16) / 2,
            terminal::size().unwrap().1 / 4
        ),
        style::SetBackgroundColor(style::Color::Black),
        style::PrintStyledContent(a.red())
    );
    io::stdout().flush();
}

pub fn announce_top_left(a: String, height: u16) {
    queue!(
        io::stdout(),
        cursor::MoveTo(2, 2 + height),
        style::SetBackgroundColor(style::Color::Black),
        style::PrintStyledContent(a.green())
    );
    io::stdout().flush();
}

pub fn player_note(a: String, height: u16) {
    queue!(
        io::stdout(),
        cursor::MoveTo(
            (terminal::size().unwrap().0 - a.len() as u16) / 2,
            terminal::size().unwrap().1 - 8 - height
        ),
        style::SetBackgroundColor(style::Color::Black),
        style::PrintStyledContent(a.red())
    );
    io::stdout().flush();
}

pub fn show_play(p: Option<&play::Play>) {
    //clear center of screen
    for i in terminal::size().unwrap().0 / 3..(terminal::size().unwrap().0 * 2) / 3 {
        for j in terminal::size().unwrap().1 / 2 - 5..terminal::size().unwrap().1 / 2 + 5 {
            queue!(
                io::stdout(),
                cursor::MoveTo(i, j),
                style::SetBackgroundColor(Color::Black),
                style::Print(" ".to_string())
            );
        }
    }

    // Only render if play exists, otherwise just clear screen
    if p.is_none() {
        return;
    }

    let play = p.unwrap();

    let play_visual_width: u16 = (1 + 3 * (play.cards.len())).try_into().unwrap();

    // white background spill
    for i in 0..play_visual_width {
        for j in terminal::size().unwrap().1 / 2 - 3..terminal::size().unwrap().1 / 2 + 3 {
            queue!(
                io::stdout(),
                cursor::MoveTo((terminal::size().unwrap().0 - play_visual_width) / 2 + i, j),
                style::SetBackgroundColor(Color::White),
                style::Print(" ".to_string())
            );
        }
    }

    queue!(
        io::stdout(),
        cursor::MoveTo(
            (terminal::size().unwrap().0 - play_visual_width) / 2,
            terminal::size().unwrap().1 / 2 - 4
        ),
    );

    for card in &play.cards {
        match card.suit {
            // change to respective colors
            card::Suit::Spades | card::Suit::Clubs | card::Suit::Black => {
                queue!(
                    io::stdout(),
                    style::SetForegroundColor(Color::Black),
                    style::SetBackgroundColor(Color::White)
                );
            }
            card::Suit::Hearts | card::Suit::Diamonds | card::Suit::Red => {
                queue!(
                    io::stdout(),
                    style::SetForegroundColor(Color::DarkRed),
                    style::SetBackgroundColor(Color::White)
                );
            }
        }

        let rank_ltr = match card.rank {
            card::Rank::Three => "3",
            card::Rank::Four => "4",
            card::Rank::Five => "5",
            card::Rank::Six => "6",
            card::Rank::Seven => "7",
            card::Rank::Eight => "8",
            card::Rank::Nine => "9",
            card::Rank::Ten => "10",
            card::Rank::Jack => "J",
            card::Rank::Queen => "Q",
            card::Rank::King => "K",
            card::Rank::Ace => "A",
            card::Rank::Two => "2",
            card::Rank::Joker => "JOK",
        };

        let suit_str = match card.suit {
            card::Suit::Spades => "♠",
            card::Suit::Diamonds => "♦",
            card::Suit::Clubs => "♣",
            card::Suit::Hearts => "♥",
            card::Suit::Red | card::Suit::Black => "",
        };
        let start_col = cursor::position().unwrap().0;
        queue!(io::stdout(), cursor::SavePosition, cursor::MoveDown(1));
        let card_display = [
            "╭──".to_string(),
            format!("│{rank_ltr}"),
            format!("│{suit_str} "),
            format!("│   "),
            format!("│   "),
            format!("╰──"),
        ];
        for str in card_display {
            queue!(
                io::stdout(),
                style::Print(str),
                cursor::MoveDown(1),
                cursor::MoveToColumn(start_col)
            );
        }
        queue!(io::stdout(), cursor::RestorePosition, cursor::MoveRight(3));
    }
    let start_col = cursor::position().unwrap().0;
    let card_display = [
        "────╮".to_string(),
        "    │".to_string(),
        "    │".to_string(),
        "    │".to_string(),
        "    │".to_string(),
        "────╯".to_string(),
    ];
    for str in card_display {
        queue!(
            io::stdout(),
            cursor::MoveDown(1),
            style::Print(str),
            cursor::MoveToColumn(start_col)
        );
    }

    let play_rank_str = &play.rank;
    let play_class_str = &play.class;
    let player = &play.player;
    queue!(
        io::stdout(),
        style::SetBackgroundColor(Color::Cyan),
        cursor::MoveUp(4),
        cursor::MoveRight(2),
        cursor::SavePosition,
        style::Print(format!("Rank: {:?}", play_rank_str)),
        cursor::RestorePosition,
        cursor::MoveDown(1),
        style::Print(format!("Class: {:?}", play_class_str)),
        cursor::RestorePosition,
        cursor::MoveDown(2),
        style::Print(format!("Player: {:?}", player.name))
    );

    io::stdout().flush();
}

//TODO maybe see if for selected cards, we can use references instead of indices
pub fn show_hand(hand: &hand::Hand, selected: &Vec<bool>, selector: usize) {
    //clear bottom of screen
    for i in 0..terminal::size().unwrap().0 {
        for j in 1..10 {
            queue!(
                io::stdout(),
                cursor::MoveTo(i, terminal::size().unwrap().1 - j),
                style::SetBackgroundColor(Color::Black),
                style::Print(" ".to_string())
            );
        }
    }

    let num_selected = selected.iter().filter(|&b| *b).count();

    let hand_visual_width: u16 = (1 + 3 * (hand.cards.len() - num_selected) + 8 * num_selected)
        .try_into()
        .unwrap();

    // white background spill
    for i in 0..hand_visual_width {
        for j in 0..4 {
            queue!(
                io::stdout(),
                cursor::MoveTo(
                    (terminal::size().unwrap().0 - hand_visual_width) / 2 + i,
                    terminal::size().unwrap().1 - j
                ),
                style::SetBackgroundColor(Color::White),
                style::Print(" ".to_string())
            );
        }
    }

    queue!(
        io::stdout(),
        cursor::MoveTo(
            (terminal::size().unwrap().0 - hand_visual_width) / 2,
            terminal::size().unwrap().1 - 4
        ),
    );

    for i in 0..hand.cards.len() {
        if usize::from(selector) == i {
            queue!(
                io::stdout(),
                cursor::SavePosition,
                style::SetForegroundColor(Color::Yellow),
                style::SetBackgroundColor(Color::Black),
                cursor::MoveUp(1),
                cursor::MoveRight(if selected[selector] { 4 } else { 1 }),
                style::Print("▼".to_string()),
                cursor::RestorePosition
            );
        }

        match hand.cards[i].suit {
            // change to respective colors
            card::Suit::Spades | card::Suit::Clubs | card::Suit::Black => {
                queue!(
                    io::stdout(),
                    style::SetForegroundColor(Color::Black),
                    style::SetBackgroundColor(Color::White)
                );
            }
            card::Suit::Hearts | card::Suit::Diamonds | card::Suit::Red => {
                queue!(
                    io::stdout(),
                    style::SetForegroundColor(Color::DarkRed),
                    style::SetBackgroundColor(Color::White)
                );
            }
        }

        let rank_ltr = match hand.cards[i].rank {
            card::Rank::Three => "3",
            card::Rank::Four => "4",
            card::Rank::Five => "5",
            card::Rank::Six => "6",
            card::Rank::Seven => "7",
            card::Rank::Eight => "8",
            card::Rank::Nine => "9",
            card::Rank::Ten => "10",
            card::Rank::Jack => "J",
            card::Rank::Queen => "Q",
            card::Rank::King => "K",
            card::Rank::Ace => "A",
            card::Rank::Two => "2",
            card::Rank::Joker => "JOK",
        };

        let rank_name = match hand.cards[i].rank {
            card::Rank::Three => "3",
            card::Rank::Four => "4",
            card::Rank::Five => "5",
            card::Rank::Six => "6",
            card::Rank::Seven => "7",
            card::Rank::Eight => "8",
            card::Rank::Nine => "9",
            card::Rank::Ten => "10",
            card::Rank::Jack => "Jack",
            card::Rank::Queen => "Queen",
            card::Rank::King => "King",
            card::Rank::Ace => "Ace",
            card::Rank::Two => "2",
            card::Rank::Joker => "Joker",
        };

        let suit_str = match hand.cards[i].suit {
            card::Suit::Spades => "♠",
            card::Suit::Diamonds => "♦",
            card::Suit::Clubs => "♣",
            card::Suit::Hearts => "♥",
            card::Suit::Red | card::Suit::Black => "",
        };

        if selected[i] {
            let start_col = cursor::position().unwrap().0;
            queue!(io::stdout(), cursor::SavePosition);
            let card_display = [
                "╭───────╮".to_string(),
                format!("│ {rank_name}  "),
                format!("│   {suit_str}   "),
                "│        ".to_string(),
            ];
            for str in card_display {
                queue!(
                    io::stdout(),
                    style::Print(str),
                    cursor::MoveDown(1),
                    cursor::MoveToColumn(start_col)
                );
            }
            queue!(io::stdout(), cursor::RestorePosition, cursor::MoveRight(8));
        } else {
            let start_col = cursor::position().unwrap().0;
            queue!(io::stdout(), cursor::SavePosition, cursor::MoveDown(1));
            let card_display = [
                "╭──".to_string(),
                format!("│{rank_ltr}"),
                format!("│{suit_str} "),
            ];
            for str in card_display {
                queue!(
                    io::stdout(),
                    style::Print(str),
                    cursor::MoveDown(1),
                    cursor::MoveToColumn(start_col)
                );
            }
            queue!(io::stdout(), cursor::RestorePosition, cursor::MoveRight(3));
        }

        io::stdout().flush();
    }
}

pub fn cleanup() {
    terminal::disable_raw_mode();
    execute!(io::stdout(), terminal::LeaveAlternateScreen);
}

pub enum Input_States {
    Esc,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Space,
    Empty,
}
