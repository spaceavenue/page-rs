use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// User-facing actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
  LineUp,
  LineDown,
  PageUp,
  PageDown,
  HalfPageUp,
  HalfPageDown,
  GotoTop,
  GotoBottom,
  Quit,
}

const KEYMAP: &[(KeyCode, KeyModifiers, Action)] = {
  use KeyCode::*;
  use KeyModifiers as Mod;

  &[
    (Char('q'), Mod::NONE, Action::Quit),
    (Esc, Mod::NONE, Action::Quit),
    (Up, Mod::NONE, Action::LineUp),
    (Char('k'), Mod::NONE, Action::LineUp),
    (Down, Mod::NONE, Action::LineDown),
    (Char('j'), Mod::NONE, Action::LineDown),
    (Enter, Mod::NONE, Action::LineDown),
    (PageUp, Mod::NONE, Action::PageUp),
    (Char('b'), Mod::CONTROL, Action::PageUp),
    (PageDown, Mod::NONE, Action::PageDown),
    (Char(' '), Mod::NONE, Action::PageDown),
    (Char('f'), Mod::CONTROL, Action::PageDown),
    (Char('u'), Mod::CONTROL, Action::HalfPageUp),
    (Char('d'), Mod::CONTROL, Action::HalfPageDown),
    (Char('g'), Mod::NONE, Action::GotoTop),
    (Home, Mod::NONE, Action::GotoTop),
    (Char('G'), Mod::NONE, Action::GotoBottom),
    (End, Mod::NONE, Action::GotoBottom),
  ]
};

pub fn action_for_key(key: KeyEvent) -> Option<Action> {
  let mods = match key.code {
    KeyCode::Char(_) => key.modifiers - KeyModifiers::SHIFT,
    _ => key.modifiers,
  };

  KEYMAP
    .iter()
    .find(|(code, map_mods, _)| *code == key.code && *map_mods == mods)
    .map(|(.., action)| *action)
}
