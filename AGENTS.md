# Agent Guidelines for Laminar

## Build/Test Commands
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test (e.g., `cargo test test_telemetry_single_insert`)
- `cargo run` - Build and run the main application

## Code Style Guidelines
- **Language**: Rust 2021 edition
- **Async**: Use `tokio` for async operations with `#[tokio::main]` and `#[tokio::test]`
- **Imports**: Group `std` first, then external crates, then local modules with blank lines between groups
- **Naming**: Use `snake_case` for variables/functions, `PascalCase` for types/structs
- **Types**: Use explicit types for clarity (e.g., `as u64`, `Duration::from_secs(10)`)
- **Error Handling**: Use `Result<(), Box<dyn std::error::Error>>` for main functions, propagate errors with `?`
- **Constants**: Use `UPPER_SNAKE_CASE` and document purpose (see `SAMPLE_CLOCK_PERIOD_NS`)
- **Comments**: Add explanatory comments for complex calculations and business logic
- **Testing**: Place tests in `#[cfg(test)]` modules, use descriptive test names, use `:memory:` for test databases
- **Spawning**: Use `tokio::spawn` for concurrent tasks, store handles when needed
- **Database**: Use `EntryJournal::initialize()` pattern for database connections
- **Match**: Use `match` for enum handling, include catch-all arms for unhandled variants
- **Printing**: Use descriptive output messages that include context and values