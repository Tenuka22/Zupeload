set shell := ["bash", "-cu"]
set windows-shell := ["powershell", "-Command"]

server-dev:
    cd server; cargo run
