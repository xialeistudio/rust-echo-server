# Rust Echo Server

An echo server implemented by Rust.

Using channel instead of shared-memory to communicate.

## Thread Model
+ Main Thread(1): handle events(`Connect`,`Disconnect`,`Frame`)
+ Listener Thread(1): accept new connections
+ Reader/Writer thread(2/connection): handle socket events