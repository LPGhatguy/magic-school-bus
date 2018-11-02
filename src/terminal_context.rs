use crossterm::{
    Crossterm,
    Screen,
};

pub struct TerminalContext<'a> {
    pub crossterm: &'a Crossterm,
    pub screen: &'a Screen,
}