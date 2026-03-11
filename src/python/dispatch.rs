use crate::board::Board;
use crate::game::Game;

/// Generates the cartesian product of W and H ranges, then invokes $mac with all (W, H) pairs.
macro_rules! cartesian_dispatch {
    ($mac:ident, [$($w:literal),*], $hs:tt) => {
        cartesian_dispatch!(@acc $mac, $hs, [] ; $($w),*);
    };
    // Base case: no more W values, invoke the target macro with accumulated pairs.
    (@acc $mac:ident, $hs:tt, [$($pairs:tt)*] ; ) => {
        $mac!($($pairs)*);
    };
    // Recursive case: peel off one W, expand all H for it, then continue.
    (@acc $mac:ident, [$($h:literal),*], [$($pairs:tt)*] ; $w:literal $(, $rest:literal)*) => {
        cartesian_dispatch!(@acc $mac, [$($h),*], [$($pairs)* $(($w, $h),)*] ; $($rest),*);
    };
}

macro_rules! define_wh_dispatch {
    ($(($w:literal, $h:literal)),* $(,)?) => {
        paste::paste! {
            #[derive(Clone)]
            pub(super) enum GameInner {
                $( [<W $w H $h>](Game<$w, $h>), )*
            }

            #[derive(Clone)]
            pub(super) enum BoardInner {
                $( [<W $w H $h>](Board<$w, $h>), )*
            }

            macro_rules! dispatch_game {
                ($self_:expr, $g:ident => $body:expr) => {
                    match $self_ {
                        $( GameInner::[<W $w H $h>]($g) => $body, )*
                    }
                };
            }

            macro_rules! dispatch_board {
                ($self_:expr, $b:ident => $body:expr) => {
                    match $self_ {
                        $( BoardInner::[<W $w H $h>]($b) => $body, )*
                    }
                };
            }

            pub(super) fn make_game_inner(width: usize, height: usize, fen: &str, castling_enabled: bool) -> Result<GameInner, String> {
                match (width, height) {
                    $( ($w, $h) => Ok(GameInner::[<W $w H $h>](Game::new(fen, castling_enabled)?)), )*
                    _ => Err(format!("Unsupported board size: {}x{}", width, height)),
                }
            }

            pub(super) fn make_standard_game_inner() -> GameInner {
                GameInner::W8H8(Game::standard())
            }

            pub(super) fn make_board_inner(width: usize, height: usize, fen: &str) -> Result<BoardInner, String> {
                match (width, height) {
                    $( ($w, $h) => Ok(BoardInner::[<W $w H $h>](Board::new(fen)?)), )*
                    _ => Err(format!("Unsupported board size: {}x{}", width, height)),
                }
            }

            pub(super) fn make_empty_board_inner(width: usize, height: usize) -> Result<BoardInner, String> {
                match (width, height) {
                    $( ($w, $h) => Ok(BoardInner::[<W $w H $h>](Board::empty())), )*
                    _ => Err(format!("Unsupported board size: {}x{}", width, height)),
                }
            }

            pub(super) fn make_standard_board_inner() -> BoardInner {
                BoardInner::W8H8(Board::standard())
            }
        }
    }
}

cartesian_dispatch!(
    define_wh_dispatch,
    [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
);
