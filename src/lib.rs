extern crate cpython;
use chess::{Board, BoardStatus, ChessMove, MoveGen, Piece, Square, EMPTY};
use std::iter::zip;
use std::str::FromStr;

use cpython::{py_fn, py_module_initializer, PyResult, Python};

py_module_initializer!(rust_bot, |py, m| {
    m.add(
        py,
        "interact_with_python",
        py_fn!(py, interact_with_python(fen: &str, depth: i8)),
    )?;
    Ok(())
});

fn best_move(board: Board, depth: i8) -> ChessMove {
    let board = board;

    let mut best_score = -isize::pow(10, 9);
    let mut alpha = -isize::pow(10, 8);
    let beta = isize::pow(10, 8);
    let mut best_move = ChessMove::new(Square::A1, Square::A2, None);

    let legal_moves = MoveGen::new_legal(&board);

    for mov in legal_moves {
        let move_score = -score_move(board.make_move_new(mov), depth - 1, -beta, -alpha);

        // println!("move: {}, score: {}", mov, move_score);

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
    println!("best score: {}", best_score);
    println!("best move: {}", best_move);
    println!("depth: {}", depth);

    best_move
}

fn score_move(board: Board, depth: i8, mut alpha: isize, beta: isize) -> isize {
    if depth == 0 {
        return evaluate_only_stable(board, alpha, beta, 100);
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

fn evaluate_only_stable(board: Board, mut alpha: isize, beta: isize, depth: i8) -> isize {
    let normal_eval = evaluate_position(board, depth);

    if depth == -100 {
        // can be made larger to not check all captures
        return normal_eval;
    }

    if normal_eval >= beta {
        return beta;
    }

    if alpha < normal_eval {
        alpha = normal_eval
    }

    // carpture moves
    let mut legal_captures = MoveGen::new_legal(&board);
    let targets = board.color_combined(!board.side_to_move());
    legal_captures.set_iterator_mask(*targets);

    // loop over all capture moves
    for mov in legal_captures {
        let score = -evaluate_only_stable(board.make_move_new(mov), -beta, -alpha, depth - 1);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha

    // todo: loop over capture moves if feasable.
}

fn evaluate_position(board: Board, depth: i8) -> isize {
    // todo: move out of check, check game over
    if board.status() != BoardStatus::Ongoing {
        return evaluate_game_over(board, depth);
    }
    if *board.checkers() != EMPTY {
        return evaluate_out_of_check(board, depth);
    }

    // count pieces default scores
    let mut score = 0;
    score += score_pieces_count(board) * 100;
    score += score_moves_count(board);
    score
}

fn evaluate_game_over(board: Board, depth: i8) -> isize {
    match board.status() {
        BoardStatus::Checkmate => -isize::pow(10, 7) + depth as isize,
        BoardStatus::Stalemate => 0,
        BoardStatus::Ongoing => unreachable!("Should already not be ongoing."),
    }
}

fn evaluate_out_of_check(board: Board, depth: i8) -> isize {
    let mut max_score = -isize::pow(10, 6);
    for mov in MoveGen::new_legal(&board) {
        let score = -evaluate_position(board.make_move_new(mov), depth);
        if max_score > score {
            max_score = score;
        }
    }
    max_score
}

/// Counts delta of legal moves.
/// positive for player on move, negative for opponent.
fn score_moves_count(board: Board) -> isize {
    let mut score = MoveGen::new_legal(&board).len() as isize;

    // Should always give something since if we're in check we should never get here.
    let reversed_board = board.null_move().expect("in check while evaluating score.");

    score -= MoveGen::new_legal(&reversed_board).len() as isize;
    score
}

/// Counts pieces delta with default score per piece.
/// Positive for player on move, negative for opponent.
fn score_pieces_count(board: Board) -> isize {
    let mut score = 0;
    let color_on_move = board.side_to_move();

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
            (board.color_combined(!color_on_move) & board.pieces(piece)).popcnt() as isize * value;
    }
    score
}

/// Gets fen string from python and returns chosen move as string.
fn interact_with_python(_py: Python, fen: &str, depth: i8) -> PyResult<String> {
    let board = Board::from_str(fen).unwrap();
    let chosen_move = best_move(board, depth);

    let move_string = format!(
        "{};{};{:?}",
        chosen_move.get_source(),
        chosen_move.get_dest(),
        chosen_move.get_promotion()
    );
    Ok(move_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_FENS: [&str; 5] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "3r2k1/5ppp/1r6/8/8/1Q6/5PPP/6K1 w - - 0 1",
        "5qk1/5ppp/8/8/3N4/8/5PPP/5QK1 w - - 0 1",
        "4k3/8/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
        "6k1/5pbp/6p1/8/8/8/3PP3/4K3 b - - 0 1",
    ];

    #[test]
    fn test_count_score() {
        let expected_outputs = [0, -1, 3, 39, 4];
        for (fen, out) in zip(TEST_FENS, expected_outputs) {
            let board = Board::from_str(fen).unwrap(); //starting position.
            let score = score_pieces_count(board);
            assert_eq!(score, out);
        }
    }

    #[test]
    fn test_moves_score() {
        let expected_outputs = [0, -2, 8, 15, 9];
        for (fen, out) in zip(TEST_FENS, expected_outputs) {
            let board = Board::from_str(fen).unwrap(); //starting position.
            let score = score_moves_count(board);
            assert_eq!(score, out);
        }
    }

    #[test]
    #[should_panic]
    fn test_count_moves_in_check() {
        let board = Board::from_str("8/8/3r4/5k2/8/3K4/8/8 w - - 0 1").unwrap(); // check position
        score_moves_count(board);
    }

    #[test]
    fn test_best_move_depth_1() {
        let fens = [
            "8/8/8/5k2/2K5/8/8/2Q4r b - - 0 1",
            "8/6p1/7p/5kn1/8/PP6/QP6/K7 w - - 0 1",
            "6k1/8/1q5P/8/8/8/1Q1K4/8 w - - 0 1",
        ];
        let outs = ["h1c1", "a2b1", "b2g7"].map(|mov| ChessMove::from_str(mov).unwrap());

        for (fen, out) in zip(fens, outs) {
            let board = Board::from_str(fen).unwrap();
            assert_eq!(best_move(board, 1), out);
        }
    }
}
