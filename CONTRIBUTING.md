# Contributing to WP Agent

Thank you for your interest in contributing to WP Agent! We welcome all contributions, from bug reports and feature requests to code changes and documentation improvements.

## Getting Started

### Prerequisites

- **Rust**: Ensure you have Rust and Cargo installed. You can install them via [rustup.rs](https://rustup.rs/).
- **WP-CLI**: While the tool can install a local copy, having `wp-cli` installed globally is helpful for testing.
- **Git**: For version control.

### Setting Up the Environment

1.  **Clone the repository**:

    ```bash
    git clone https://github.com/yourusername/wp-agent-tool.git
    cd wp-agent-tool
    ```

2.  **Build the project**:

    ```bash
    cargo build
    ```

3.  **Run the tool**:
    ```bash
    cargo run
    ```

## Development Workflow

1.  **Create a Branch**: Always create a new branch for your changes.

    ```bash
    git checkout -b feature/my-new-feature
    ```

2.  **Make Changes**: Implement your feature or fix.

3.  **Run Checks**: Ensure your code compiles and passes checks.

    ```bash
    cargo check
    cargo test
    ```

4.  **Format Code**: We use `rustfmt` to keep the code consistent.

    ```bash
    cargo fmt
    ```

5.  **Commit**: Use descriptive commit messages.

6.  **Push and Open a PR**: Push your branch to GitHub and open a Pull Request.

## Project Structure

- `src/main.rs`: Entry point. Initializes the CLI and runs the diagnosis modules.
- `src/wp.rs`: Wrapper for executing `wp-cli` commands.
- `src/report.rs`: Handles the formatted output of the diagnosis results.
- `src/diagnosis/`: Directory containing all diagnosis logic.
  - `mod.rs`: Registers the diagnosis modules.
  - `database.rs`, `plugins.rs`, etc.: Individual modules.

## How to Add a New Diagnosis Module

1.  **Create the Module File**:
    Create a new file in `src/diagnosis/` (e.g., `my_new_check.rs`).

2.  **Implement the `Diagnosis` Trait**:
    Your struct must implement the `Diagnosis` trait.

    ```rust
    use crate::diagnosis::{Diagnosis, DiagnosisReport, Status};
    use crate::wp::WpCli;
    use std::path::Path;
    use anyhow::Result;

    pub struct MyNewCheck;

    impl Diagnosis for MyNewCheck {
        fn run(&self, wp: &WpCli, root: &Path) -> Result<DiagnosisReport> {
            // Your logic here
            Ok(DiagnosisReport {
                module: "My New Check".to_string(),
                status: Status::Ok,
                message: "Everything looks good".to_string(),
                details: vec![],
            })
        }
    }
    ```

3.  **Register the Module**:
    - Add `pub mod my_new_check;` to `src/diagnosis/mod.rs`.
    - In `src/main.rs`, add the module to the `modules` vector:
      ```rust
      use diagnosis::my_new_check::MyNewCheck;

      // ... inside main()
      let modules: Vec<Box<dyn Diagnosis>> = vec![
          // ...
          Box::new(MyNewCheck),
      ];
      ```

## License

By contributing, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
