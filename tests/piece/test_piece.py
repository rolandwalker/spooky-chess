import pytest

import spooky_chess


def test_piece_creation() -> None:
    white_king = spooky_chess.Piece("k", spooky_chess.WHITE)
    assert white_king.color() == spooky_chess.WHITE
    assert white_king.symbol() == "K"

    black_pawn = spooky_chess.Piece("p", spooky_chess.BLACK)
    assert black_pawn.color() == spooky_chess.BLACK
    assert black_pawn.symbol() == "p"


def test_invalid_piece_types() -> None:
    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("invalid", spooky_chess.WHITE)

    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("", spooky_chess.WHITE)


def test_invalid_colors() -> None:
    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("k", 0)

    with pytest.raises(ValueError):  # noqa: PT011
        spooky_chess.Piece("k", 2)
