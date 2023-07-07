# db_new_record_to_json

Directory "/add" (included in ".gitignore") contains: "last_accept_time", "mempool.sqlite", "output.json"

Last accept time example "add/last_accept_time":
```bash
1687841601
```
If the file is empty, missing, or cannot be read, it will be overwritten, variable last_accept_time will be 0, and the program will reindex the database.

Run with path to your data base (necessary):
```bash
RUST_LOG=debug cargo run -- add/mempool.sqlite
```
Run with path to outpun json (optional):
```bash
RUST_LOG=debug cargo run -- add/mempool.sqlite -o add/output.json
```
Run with path to last_accept_time (optional):
```bash
RUST_LOG=debug cargo run -- add/mempool.sqlite -t add/last_accept_time
```
Replace "add' with path to your directory. 

If you didn't provide this argument, output.json would be created in "/add' directory by default. 

Run with logger:
```bash
RUST_LOG=debug cargo run -- add/mempool.sqlite
RUST_LOG=debug cargo run -- add/mempool.sqlite -o add/output.json
```

Build and run:
```bash
cargo build
RUST_LOG=debug ./target/debug/db_to_json_1 add/mempool.sqlite
RUST_LOG=debug ./target/debug/db_to_json_1 add/mempool.sqlite -o add/output.json
RUST_LOG=debug ./target/debug/db_to_json_1 add/mempool.sqlite -o add/output.json -t add/last_accept_time
```
