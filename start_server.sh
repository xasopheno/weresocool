echo "Starting weresovisible and ws_server...\n"
((cd ../wereso_visible && yarn start) & (cd ../ws/server && cargo run --release))
