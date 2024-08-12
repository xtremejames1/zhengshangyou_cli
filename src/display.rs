use crate::card;
use crate::hand;
use crate::play;
use crate::player;

use crossterm::cursor::SetCursorStyle;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::style::SetForegroundColor;
use crossterm::{
    cursor, execute, queue,
    style::{self, Color, Stylize},
    terminal,
};
use std::any::Any;
use std::collections::VecDeque;
use std::io::stdout;
use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

pub trait Renderable: Any + Send {
    fn render_init(&self) -> Result<(), &'static str>;
    fn render_update(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
    fn as_any(&mut self) -> &mut dyn Any;
    // Whether the renderable should be destroyed
    fn destroy(&mut self) -> bool {
        false
    }
}

pub struct Display {
    pub renderables: VecDeque<Arc<Mutex<dyn Renderable>>>,
}

impl Display {
    pub fn new() -> Self {
        terminal::enable_raw_mode().expect("Failed to enable raw_mode");
        queue!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)
            .expect("Failed creating new display");
        io::stdout().flush().expect("Failed to write to stdout");
        Self {
            renderables: VecDeque::new(),
        }
    }

    pub fn add_renderable<T>(&mut self, renderable: Arc<Mutex<T>>)
    where
        T: Renderable + 'static,
    {
        renderable
            .lock()
            .unwrap()
            .render_init()
            .expect("Initializing renderable failed");

        // Cast Arc<Mutex<T>> to Arc<Mutex<dyn Renderable>>
        let dyn_renderable: Arc<Mutex<dyn Renderable>> = renderable;
        self.renderables.push_back(dyn_renderable);
    }

    pub fn update(&mut self) {
        // Remove all renderables which need to be destroyed
        self.renderables
            .retain(|renderable| !renderable.lock().unwrap().destroy());
        for renderable in &self.renderables {
            renderable
                .lock()
                .unwrap()
                .render_update()
                .expect("Rendering renderable failed");
        }
    }
}

pub struct CheckBox {
    prompt: String,
    pub checked: bool,
}

impl CheckBox {
    pub fn new<T>(prompt: T) -> Self
    where
        T: Into<String>,
    {
        let prompt = prompt.into();
        Self {
            prompt,
            checked: false,
        }
    }
}

impl Renderable for CheckBox {
    fn render_init(&self) -> Result<(), &'static str> {
        queue!(
            io::stdout(),
            style::SetBackgroundColor(Color::Black),
            style::SetForegroundColor(Color::White),
            cursor::MoveTo(
                ((terminal::size().unwrap().0 as usize - &self.prompt.len()) / 2)
                    .try_into()
                    .unwrap(),
                terminal::size().unwrap().1 / 2 - 1
            ),
            style::Print(&self.prompt),
            cursor::MoveTo(
                (terminal::size().unwrap().0 / 2).try_into().unwrap(),
                terminal::size().unwrap().1 / 2
            ),
            style::Print(""),
        )
        .expect("Failed to queue io changes");
        io::stdout().flush().expect("Failed to write to stdout");
        Ok(())
    }

    fn render_update(&mut self) -> Result<(), &'static str> {
        let mut updated = true;
        //Check keystroke
        if poll(Duration::from_millis(500)).ok().unwrap() {
            let event = read();
            match event {
                Ok(Event::Key(event)) if event.kind == KeyEventKind::Press => match event.code {
                    KeyCode::Enter => {
                        self.checked = !self.checked;
                        queue!(io::stdout(), cursor::Hide).expect("Failed to queue io changes");
                    }
                    _ => {
                        updated = false;
                    }
                },
                _ => {
                    updated = false;
                }
            }
        }

        if updated {
            let mut middle = String::new();
            for _ in 0..terminal::size().unwrap().0 / 2 - 2 {
                middle.push_str(" ");
            }
            queue!(
                io::stdout(),
                style::SetBackgroundColor(Color::Black),
                cursor::MoveTo(
                    (terminal::size().unwrap().0 / 2).try_into().unwrap(),
                    terminal::size().unwrap().1 / 2
                ),
                style::Print(if self.checked { "󰄲" } else { "" }),
            )
            .expect("Failed to queue io changes");
            io::stdout().flush().expect("Failed to write to stdout");
        }
        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct InputBox {
    prompt: String,
    cursor: usize,
    current_input: String,
    pub output: Option<String>,
}

impl InputBox {
    //todo create thread that reads input when selected

    pub fn new<T>(prompt: T) -> Self
    where
        T: Into<String>,
    {
        let prompt = prompt.into();
        Self {
            current_input: String::new(),
            cursor: 0,
            prompt,
            output: None,
        }
    }
}

impl Renderable for InputBox {
    fn render_init(&self) -> Result<(), &'static str> {
        let mut top_border = "╭".to_string();
        let mut bottom_border = "╰".to_string();
        let mut middle = String::new();
        for _ in 0..terminal::size().unwrap().0 / 2 - 2 {
            top_border.push_str("─");
            bottom_border.push_str("─");
            middle.push_str(" ");
        }
        top_border.push_str("╮");
        bottom_border.push_str("╯");
        queue!(
            io::stdout(),
            style::SetBackgroundColor(Color::Black),
            style::SetForegroundColor(Color::White),
            cursor::MoveTo(
                terminal::size().unwrap().0 / 4,
                terminal::size().unwrap().1 / 2 - 1
            ),
            style::Print(top_border),
            cursor::MoveTo(
                terminal::size().unwrap().0 / 4 + 1,
                terminal::size().unwrap().1 / 2
            ),
            style::Print(middle),
            cursor::MoveTo(
                terminal::size().unwrap().0 / 4,
                terminal::size().unwrap().1 / 2 + 1
            ),
            style::Print(bottom_border),
            cursor::MoveTo(
                terminal::size().unwrap().0 / 4,
                terminal::size().unwrap().1 / 2
            ),
            style::Print("│"),
            cursor::MoveTo(
                terminal::size().unwrap().0 * 3 / 4 - 1,
                terminal::size().unwrap().1 / 2
            ),
            style::Print("│"),
            cursor::MoveTo(
                ((terminal::size().unwrap().0 as usize - &self.prompt.len()) / 2)
                    .try_into()
                    .unwrap(),
                terminal::size().unwrap().1 / 2 - 1
            ),
            style::Print(&self.prompt),
            cursor::MoveTo(
                terminal::size().unwrap().0 / 2,
                terminal::size().unwrap().1 / 2
            ),
            SetCursorStyle::BlinkingBar,
            cursor::Show,
        )
        .expect("Failed to queue io changes");
        io::stdout().flush().expect("Failed to write to stdout");
        Ok(())
    }

    fn render_update(&mut self) -> Result<(), &'static str> {
        let mut updated = true;
        //Check keystroke
        if poll(Duration::from_millis(500)).ok().unwrap() {
            let event = read();
            match event {
                Ok(Event::Key(event)) if event.kind == KeyEventKind::Press => match event.code {
                    KeyCode::Esc => {
                        self.output = Some("\0".into());
                        queue!(io::stdout(), cursor::Hide).expect("Failed to queue io changes");
                    }
                    KeyCode::Left => {
                        if self.cursor != 0 {
                            self.cursor -= 1;
                            queue!(io::stdout(), cursor::MoveLeft(1))
                                .expect("Failed to queue io changes");
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor != self.current_input.len() {
                            self.cursor += 1;
                            queue!(io::stdout(), cursor::MoveRight(1))
                                .expect("Failed to queue io changes");
                        }
                    }
                    KeyCode::Enter => {
                        self.output = Some(self.current_input.clone());
                        queue!(io::stdout(), cursor::Hide).expect("Failed to queue io changes");
                    }
                    KeyCode::Backspace => {
                        if self.cursor > 0 {
                            self.current_input.remove(self.cursor - 1);
                            self.cursor -= 1;
                            queue!(io::stdout(), cursor::MoveLeft(1))
                                .expect("Failed to queue io changes");
                        }
                    }
                    KeyCode::Char(char) => {
                        self.current_input.insert(self.cursor, char);
                        self.cursor += 1;
                        queue!(io::stdout(), cursor::MoveRight(1))
                            .expect("Failed to queue io changes");
                    }
                    _ => {
                        updated = false;
                    }
                },
                _ => {
                    updated = false;
                }
            }
        }

        if updated {
            let mut middle = String::new();
            for _ in 0..terminal::size().unwrap().0 / 2 - 2 {
                middle.push_str(" ");
            }
            queue!(
                io::stdout(),
                style::SetBackgroundColor(Color::Black),
                cursor::MoveTo(
                    terminal::size().unwrap().0 / 4 + 1,
                    terminal::size().unwrap().1 / 2
                ),
                style::Print(middle),
                cursor::MoveTo(
                    terminal::size().unwrap().0 / 4 + 1,
                    terminal::size().unwrap().1 / 2
                ),
                style::Print(&self.current_input),
                cursor::MoveTo(
                    terminal::size().unwrap().0 / 4 + 1 + self.cursor as u16,
                    terminal::size().unwrap().1 / 2
                ),
            )
            .expect("Failed to queue io changes");
            io::stdout().flush().expect("Failed to write to stdout");
        }
        Ok(())
    }

    // Destroy the input box once output is received
    fn destroy(&mut self) -> bool {
        if self.output.is_some() {
            let mut top_border = String::new();
            let mut bottom_border = String::new();
            let mut middle = String::new();
            for _ in 0..terminal::size().unwrap().0 / 2 {
                top_border.push_str(" ");
                bottom_border.push_str(" ");
                middle.push_str(" ");
            }
            queue!(
                io::stdout(),
                cursor::Hide,
                style::SetBackgroundColor(Color::Black),
                style::SetForegroundColor(Color::White),
                cursor::MoveTo(
                    terminal::size().unwrap().0 / 4,
                    terminal::size().unwrap().1 / 2 - 1
                ),
                style::Print(top_border),
                cursor::MoveTo(
                    terminal::size().unwrap().0 / 4,
                    terminal::size().unwrap().1 / 2
                ),
                style::Print(middle),
                cursor::MoveTo(
                    terminal::size().unwrap().0 / 4,
                    terminal::size().unwrap().1 / 2 + 1
                ),
                style::Print(bottom_border),
                cursor::MoveTo(
                    terminal::size().unwrap().0 / 4,
                    terminal::size().unwrap().1 / 2
                ),
            )
            .expect("Failed destroying Input Box");
            io::stdout().flush().expect("Failed to write to stdout");

            return true;
        }
        false
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub fn get_keystate() -> Input_States {
    if poll(Duration::from_millis(500)).ok().unwrap() {
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

pub struct Warning {
    text: String,
    timeout: Duration,
    initial_time: Instant,
}

impl Warning {
    pub fn new<T>(text: T, timeout: Duration) -> Self
    where
        T: Into<String>,
    {
        Self {
            text: text.into(),
            timeout,
            initial_time: Instant::now(),
        }
    }
}

impl Renderable for Warning {
    fn render_init(&self) -> Result<(), &'static str> {
        queue!(
            io::stdout(),
            cursor::MoveTo(
                (terminal::size().unwrap().0 - self.text.len() as u16) / 2,
                terminal::size().unwrap().1 * 3 / 4
            ),
            style::SetBackgroundColor(style::Color::DarkRed),
            style::PrintStyledContent(self.text.clone().white())
        )
        .expect("Queueing renderable failed");
        io::stdout().flush().expect("Writing to stdout failed");
        Ok(())
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    //Destroy if expired
    fn destroy(&mut self) -> bool {
        self.initial_time.elapsed().lt(&self.timeout)
    }
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

pub fn show_server_status(players_streams: &VecDeque<(player::Player, TcpStream, Instant)>) {
    for i in terminal::size().unwrap().0 / 3..(terminal::size().unwrap().0 * 2) / 3 {
        for j in terminal::size().unwrap().1 / 3..terminal::size().unwrap().1 * 2 / 3 {
            queue!(
                io::stdout(),
                cursor::MoveTo(i, j),
                style::SetBackgroundColor(Color::Black),
                style::Print(" ".to_string())
            );
        }
    }

    if players_streams.is_empty() {
        return;
    }

    queue!(
        io::stdout(),
        cursor::MoveTo(
            (terminal::size().unwrap().0) / 3 + 2,
            terminal::size().unwrap().1 / 3 + 2
        ),
    );

    for player in players_streams {
        queue!(
            io::stdout(),
            cursor::SavePosition,
            SetForegroundColor(Color::White)
        );
        let name = &player.0.name;
        let ip = &player.1.peer_addr().unwrap_or("0.0.0.0:0".parse().unwrap());
        let time = &player.2.elapsed().as_millis();
        queue!(
            io::stdout(),
            style::Print(format!("{name} - {ip} - {time}ms")),
            cursor::RestorePosition,
            cursor::MoveDown(1),
        );
    }

    stdout().flush();
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
