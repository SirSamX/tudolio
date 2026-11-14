
# tudolio

A simple, lightweight terminal-based todo manager written in Rust.

## Installation

```bash
cargo install tudolio
```

## Usage

```bash
# Add a todo
todo add "Learn Rust"

# List todos
todo list

# Mark as done
todo done 1

# Reopen a todo
todo open 1

# Edit a todo
todo edit 1 "Learn more Rust"

# Remove a todo
todo remove 1

# Clear completed todos
todo clear
```

You can also pipe input:

```bash
echo "Buy groceries" | todo add
echo "1" | todo done
```

## License

MIT
