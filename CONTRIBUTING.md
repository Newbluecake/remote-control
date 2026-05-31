# Contributing

Thanks for your interest in contributing to remote-control!

## Development Setup

```bash
git clone https://github.com/Newbluecake/remote-control.git
cd remote-control
cargo build
```

See [docs/development.md](docs/development.md) for detailed development guide.

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Ensure `cargo clippy` passes with no warnings
5. Ensure `cargo fmt` has been run
6. Commit with a descriptive message following [Conventional Commits](https://www.conventionalcommits.org/)
7. Push and open a PR

## Commit Message Format

```
type: short description

Optional longer description.
```

Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- No comments unless explaining a non-obvious "why"
- Prefer simplicity over abstraction

## Areas for Contribution

- Windows testing and bug fixes
- TLS (`wss://`) support
- P2P direct connection mode
- Better terminal UI (e.g., ratatui)
- Configuration file support
- Audio track sync
- Playlist sync
