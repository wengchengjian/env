# env - Development Environment Manager

A powerful command-line tool for managing multiple development environments efficiently. It helps developers easily install, manage, and switch between different versions of development tools and runtime environments.

## Features

- Easy installation and version management for:
  - Java (JDK)
  - Python
  - Node.js
  - Rust
  - Go
  - MySQL
  - PostgreSQL
  - MongoDB
  - Redis
  - Maven
  - Gradle
- Automatic environment variable configuration
- Simple version switching between different installations
- Global configuration management
- Interactive installation process

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/env.git
cd env

# Build the project
cargo build --release

# Add to your PATH
# The executable will be in target/release/env
```

## Usage

### Basic Commands

```bash
# Install a development environment (interactive mode)
env dev

# Install a specific environment
env dev java
env dev python
env dev node

# Switch versions for an installed environment
env choose java
env choose python
env choose node

# Configure installation directory
env config --dir "C:\Program Files\env"

# Refresh environment configuration
env config --flush

# View current configuration
env config
```

### Supported Environments

The tool currently supports the following environments:
- Java (JDK)
- Python
- Node.js
- Rust
- Go
- MySQL
- PostgreSQL
- MongoDB
- Redis
- Maven
- Gradle

Each environment can be installed and managed using the same command pattern:
```bash
env dev [environment-name]
env choose [environment-name]
```

## Configuration

The tool manages its configuration through two main files:

1. `$HOME/.env.config.json`: User configuration file that includes:
   - Installation directory path
   - Installed versions for each environment
   - Current active versions
   - Environment-specific settings

2. `.env.config.default.json`: Default configuration template that defines:
   - Available environments
   - Download repositories
   - Environment-specific configurations

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
