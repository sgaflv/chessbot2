# chessbot
Easy chess bot written in Rust

- uses 64-bit bitboards with magic multiplications
- simplistic board evaluation
- simple min-max search for a fixed 4-plies depth with alpha-beta pruning

To try it out, compile with:
cargo build --release

You will need winboard or xboard to play with it.
