# Testing Scripts

Dev scripts to run tests for the gytmdl-gui application.

## Available Scripts

### Backend Testing

#### `test-backend.sh`
Runs all Rust backend tests including data models, configuration management, and other backend components.

```bash
./scripts/test-backend.sh
```

#### `test-data-models.sh`
Specifically runs tests for the data models (state management and configuration). This is useful for regression testing the core data structures.

```bash
./scripts/test-data-models.sh
```

#### `test-watch.sh`
Rudimentary watch mode. Runs tests in watch mode for development. Tests will automatically rerun when the code changes.

```bash
./scripts/test-watch.sh
```

### 🌐 Full Test Suite

#### `test-full.sh`
Runs both frontend and backend tests (when frontend tests are available).

```bash
./scripts/test-full.sh
```

## Prerequisites

- Rust and Cargo: Required for backend tests
- Node.js and npm: Required for frontend tests (when available)
- cargo-watch: Automatically installed by `test-watch.sh` if not present

## Making Scripts Executable

Before running the scripts, make them executable:

```bash
chmod +x scripts/*.sh
```

## Test Coverage

The current test suite covers:

### Data Models (`test-data-models.sh`)
- State Management: AppState operations, job lifecycle, queue management
- Configuration: Serialization, validation, file I/O
- Thread Safety: Concurrent access to shared state
- Serialization: JSON serialization/deserialization
- Error Handling: Invalid operations, file system errors

### Test Statistics
- 29 test cases covering core functionality
- Thread safety tests with concurrent operations
- File I/O tests with temporary directories
- Serialization tests for data persistence

## Running Specific Tests

You can also run specific test modules directly:

```bash
# Run only state management tests
cd src-tauri && cargo test modules::state::tests

# Run only configuration tests  
cd src-tauri && cargo test modules::config_manager::tests

# Run a specific test
cd src-tauri && cargo test test_app_state_thread_safety
```

## Continuous Integration

These scripts are designed to be CI-friendly and will:
- Exit with non-zero code on test failures
- Provide clear output for debugging
- Build the project before running tests

## Development Workflow

For active development, use the watch mode:

```bash
./scripts/test-watch.sh
```

This will automatically run tests whenever you save changes to your Rust files, providing immediate feedback during development.