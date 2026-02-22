set shell := ["bash", "-cu"]
set windows-shell := ["powershell", "-Command"]

server-dev:
    cd server; cargo run --bin runner

server-test-gen:
    cd server; cargo run --bin test_file_generator
