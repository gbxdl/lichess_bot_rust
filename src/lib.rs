extern crate cpython;
use chess::{Board, ChessMove, Color, MoveGen, Piece, Square};
use std::iter::zip;
use std::str::FromStr;

use cpython::{py_fn, py_module_initializer, PyResult, Python};

py_module_initializer!(rust_bot, |py, m| {
    m.add(
        py,
        "interact_with_python",
        py_fn!(py, interact_with_python(fen: &str)),
    )?;
    Ok(())
});

fn best_move(board: Board) -> ChessMove {
    let depth = 6;
    let board = board;

    let mut best_score = -isize::pow(10, 9);
    let mut alpha = -(isize::pow(10, 8));
    let beta = isize::pow(10, 8);
    let mut best_move = ChessMove::new(Square::A1, Square::A2, None);

    let legal_moves = MoveGen::new_legal(&board);

    for mov in legal_moves {
        let move_score = -score_move(board.make_move_new(mov), depth - 1, -beta, -alpha);

        // update best score and best move
        if move_score > best_score {
            best_score = move_score;
            best_move = mov;
        }

        //update alpha
        if move_score > alpha {
            alpha = move_score;
        }
    }
    best_move
}

fn score_move(board: Board, depth: u8, mut alpha: isize, beta: isize) -> isize {
    if depth == 0 {
        return evaluate_only_stable(board, alpha, beta);
    }

    for mov in MoveGen::new_legal(&board) {
        let move_score = -score_move(board.make_move_new(mov), depth - 1, -beta, -alpha);

        // return beta condition
        if move_score >= beta {
            return beta;
        }

        // update alpha
        if move_score > alpha {
            alpha = move_score
        }
    }
    alpha
}

fn evaluate_only_stable(board: Board, _alpha: isize, _beta: isize) -> isize {
    let normal_eval = evaluate_position(board);
    normal_eval

    // todo: loop over capture moves if feasable.
}

fn evaluate_position(board: Board) -> isize {
    // todo: move out of check, check game over

    // count pieces default scores
    let mut score = 0;
    score += score_pieces_count(board) * 100;
    score
}

/// Counts pieces with default score per piece.
/// positive for player on move, negative for opponent.
fn score_pieces_count(board: Board) -> isize {
    let mut score = 0;
    let color_on_move = board.side_to_move();
    let other_color = match color_on_move {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    for (piece, value) in zip(
        [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
        ],
        [1, 3, 3, 5, 9],
    ) {
        score +=
            (board.color_combined(color_on_move) & board.pieces(piece)).popcnt() as isize * value;
        score -=
            (board.color_combined(other_color) & board.pieces(piece)).popcnt() as isize * value;
    }
    score
}

/// Just returns the first move as a proof of concept.
fn _first_move(board: Board) -> ChessMove {
    let legal_moves = MoveGen::new_legal(&board);
    let mut first_move = ChessMove::new(Square::B1, Square::C3, None);
    for mov in legal_moves {
        first_move = mov;
        break;
    }
    first_move
}

/// Gets fen string from python and returns chosen move as string.
fn interact_with_python(_py: Python, fen: &str) -> PyResult<String> {
    let board = Board::from_str(fen).unwrap();
    let chosen_move = best_move(board);

    let move_string = format!(
        "{};{};{:?}",
        chosen_move.get_source(),
        chosen_move.get_dest(),
        chosen_move.get_promotion()
    );
    Ok(move_string)
}

// fn get_result(_py: Python, fen: &str) -> PyResult<String> {
//     let board = Board::from_str(fen).unwrap();

//     // create an iterable
//     let legal_moves = MoveGen::new_legal(&board);

//     let mut first_move = ChessMove::new(Square::B1, Square::C3, None);

//     for mov in legal_moves {
//         first_move = mov;
//         break;
//     }

//     let move_string = format!(
//         "{};{};{:?}",
//         first_move.get_source(),
//         first_move.get_dest(),
//         first_move.get_promotion()
//     );

//     Ok(move_string)
// }
