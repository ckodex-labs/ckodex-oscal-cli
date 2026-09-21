#![allow(unused_imports)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

pub(super) fn next_screen(screen: Screen) -> Screen {
    let index = Screen::ALL
        .iter()
        .position(|candidate| *candidate == screen)
        .unwrap_or(0);
    Screen::ALL[(index + 1) % Screen::ALL.len()]
}
