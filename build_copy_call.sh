cargo build -r
cp target/release/librust_bot.dylib lichess-bot/rust_bot.so
cd lichess-bot
python3 lichess-bot.py