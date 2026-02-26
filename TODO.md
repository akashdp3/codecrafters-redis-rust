# TODO

## Correctness

- [ ] Fix expiry pipeline: change `RedisValue.expiry` in [rdb_parser.rs](cci:7://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/rdb_parser.rs:0:0-0:0) to `Option<SystemTime>`

## Idiomatic Rust

- [ ] Add `Expiry` newtype (`src/types.rs`) unifying TTL and Unix timestamp expiry
- [ ] Replace [IntoSystemTime](cci:2://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/store/db.rs:13:0-15:1) trait with methods on `Expiry`
- [ ] Implement `From<String>` / `From<&str>` for `Resp`
- [ ] Add builder pattern for [RDB](cci:2://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/rdb_parser.rs:19:0-23:1)
- [ ] Add `parse_expiry`, `parse_entry`, `parse_database` helper methods to [RDBParser](cci:2://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/rdb_parser.rs:58:0-60:1)
- [ ] Add opcode constants module in [rdb_parser.rs](cci:7://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/rdb_parser.rs:0:0-0:0)
- [ ] Add `src/error.rs` with `thiserror` domain errors

## Zero-Copy

- [ ] Change [Resp::decode](cci:1://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/resp.rs:27:4-34:5) to return `Vec<&str>` (lifetime-bound slices)
- [ ] Change [Command::parse](cci:1://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/command.rs:47:4-134:5) to accept `&[&str]`
- [ ] Refactor [RDBParser](cci:2://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/rdb_parser.rs:58:0-60:1) to read entire file upfront and use cursor-based slicing

## Structure

- [ ] Move [rdb_parser.rs](cci:7://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/rdb_parser.rs:0:0-0:0) to `src/rdb/` submodule
- [ ] Split [command.rs](cci:7://file:///Users/asam3049/projects/codecrafters/codecrafters-redis-rust/src/command.rs:0:0-0:0) into parse and execute concerns
- [x] Replace positional CLI args with `clap`
