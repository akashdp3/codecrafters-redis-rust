# Design TODO

Structural improvements from the mini-redis comparison review. Ordered by leverage: finishing #1 unblocks most of the others. Unlike `TODO.md` (which tracks line-level cleanup), items here are module-shape changes.

## 1. Split the `Store` monolith

The single `Arc<Mutex<Store>>` serializes every client and forces I/O-under-lock. Break it up.

- [ ] Introduce `Db` owning just the hashmap + expiry, guarded by `std::sync::Mutex` (never held across `.await`)
- [ ] Extract `Config` into `Arc<Config>` — immutable after startup, no lock
- [ ] Remove `replicas: Vec<ReplicaState>` from `Store` (moves to the replication actor in #2)
- [ ] Replace `Arc<Mutex<Store>>` plumbing in `main.rs`, `handler/mod.rs`, every `command/*::invoke` with the narrower handles
- [ ] Delete `tokio::sync::Mutex` usage once no lock spans an await

**Reference:** `mini-redis/src/db.rs` — `Shared { state: Mutex<State>, background_task: Notify }`

## 2. Replication actor

Replication logic is currently smeared across `handler::sync`, `Command::Wait`, and `Store`. Consolidate it into one task owning all replica sockets.

- [ ] Define `ReplicationHandle { tx: mpsc::Sender<ReplMsg> }`
- [ ] Messages: `Propagate(Bytes)`, `RegisterReplica(Conn)`, `WaitAck { numreplicas, timeout, reply: oneshot::Sender<usize> }`
- [ ] Move `handle_replication` + replica socket ownership into the actor
- [ ] Replace `handler::sync` re-match with `cmd.propagates() -> Option<Resp>` on the command enum
- [ ] Rewrite `command::wait::invoke` to send `WaitAck` and await the oneshot — no more locking the store to poll replica sockets

## 3. Unify RESP parsing

Frame boundaries are currently parsed twice: once in `Conn::find_frame_end` to size the frame, once in `Resp::decode` to extract values. One source of truth.

- [ ] Make `Resp` the real frame type (keep the enum, extend decode side to preserve `Integer`/nested `Array`)
- [ ] Add `Resp::check(&mut Cursor<&[u8]>) -> Result<(), Incomplete>`
- [ ] Add `Resp::parse(&mut Cursor<&[u8]>) -> Result<Resp>`
- [ ] Rewrite `Conn::read_frame` as `check → split_to → parse`
- [ ] Delete `Conn::find_frame_end`
- [ ] Change `Resp::decode` to return `Resp` (not `Vec<String>`); update `Command::parse` to pattern-match on `Resp::Array(..)`

**Reference:** `mini-redis/src/frame.rs`, `src/connection.rs`

## 4. Move dispatch onto per-command structs

`Command::execute` is a 50-line match where every arm locks the store differently. Each command should own its own execution.

- [ ] Convert each enum variant to a struct: `Command::Get(Get)`, `Command::Set(Set)`, …
- [ ] Implement `apply` per struct, taking only the dependencies that command needs (`&Db`, `&Arc<Config>`, `&ReplicationHandle`)
- [ ] Return `Resp` from `apply`, not `Vec<u8>` — let the connection encode
- [ ] Handle `Psync`'s RDB blob via a `Resp::Raw(Bytes)` or `Resp::Pipelined(Vec<Resp>)` variant instead of leaking bytes into the return type
- [ ] Collapse `Command::execute` to a three-line delegation match

**Reference:** `mini-redis/src/cmd/mod.rs`

## 5. Fix dependency direction

`store` currently imports `Conn` from `server` because `ReplicaState` holds a live socket. Storage should not know about transport.

- [ ] After #2, `Store`/`Db` holds no `Conn`
- [ ] Remove `use crate::server::*` from anything under `src/store/`
- [ ] Verify: `cargo-modules` or manual check — `store` imports nothing from `server` or `handler`

## 6. Typed error taxonomy

Today a "wrong args for SET" and an `io::Error` both become `-ERR <anyhow chain>` back to the client. Separate protocol errors (→ RESP) from internal errors (→ log + close).

- [ ] Add `src/error.rs` with `thiserror`:
  - `Error::Protocol(String)` → serialize as `Resp::SimpleError`
  - `Error::Io(#[from] io::Error)` → log, close connection
  - `Error::Rdb(RdbError)` → log at startup, don't bubble to clients
- [ ] `Command::parse` returns `Result<Command, ProtocolError>`
- [ ] Keep `anyhow::Result` at `main.rs` boundary only

## 7. Hygiene

- [ ] Delete `Conn::_clear_buffer` and `Conn::_write_frame` (underscore-prefixed dead code)
- [ ] Replace `IntoSystemTime` trait (`src/store/db.rs:38`) with an `Expiry` newtype — already in `TODO.md`, do it alongside #1
- [ ] Replace `Rdb::new() + header() + metadata() + data()` setter chain (`src/rdb_parser.rs:26-52`) with direct field assignment in the parse loop
- [ ] Once #3 lands, change `Command::parse` to take `&Resp` instead of `Vec<String>` and drop the `Iterator<Item = String>` plumbing in every `command/*::parse`
