# mamod

Matrix moderation tools

## Description

`mamod` is a simple Rust-based tool for Matrix server moderation. It lists all contacts who are in the rooms of a specified Matrix user.

## Requirements

- Rust 1.91+ (for building)
- [synadm](https://github.com/JOJ0/synadm) - configured and working with JSON output

## Building

```bash
cargo build --release
```

The binary will be available at `target/release/mamod`.

## Usage

```bash
mamod <matrix_username>
```

### Example

```bash
mamod @alice:example.com
```

This will:
1. Get all rooms for the user `@alice:example.com`
2. Get all members from each room
3. Deduplicate the contacts
4. Display a sorted list of all contacts (excluding the queried user)

### Output

```
Contacts for user @alice:example.com:
@bob:example.com
@charlie:example.com
@dave:example.com
```

## How it works

The tool uses the `synadm` command-line interface to interact with the Matrix server:

1. Calls `synadm user rooms <username>` to retrieve all rooms for the specified user
2. For each room, calls `synadm room members <room_id>` to get the room members
3. Collects all unique contacts (excluding the queried user)
4. Displays the results in a sorted, alphabetical order

## Design Philosophy

This tool follows the KISS (Keep It Simple, Stupid) principle:
- Minimal dependencies (only serde and serde_json for JSON parsing)
- Straightforward implementation
- Clear error messages
- No unnecessary features

## License

See LICENSE file for details.
