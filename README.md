# env - Development Environment Manager

A command-line tool for managing multiple development environments efficiently.

## Features

- Easy installation and version management for:
  - Java (JDK)
  - Python (Coming soon)
  - Node.js (Coming soon)
  - Rust (Coming soon)
  - Go (Coming soon)
  - MySQL (Coming soon)
  - Redis (Coming soon)
- Automatic environment variable configuration
- Simple version switching between different installations
- Global configuration management

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
# Install a specific environment
env dev java

# Install all supported environments
env dev --all

# Switch versions
env choose java

# Configure installation directory
env config --dir "C:\Program Files\env"

# Refresh environment configuration
env config --flush
```

### Java Environment Management

```bash
# Install specific Java version
env dev java -v 17.0.9

# Switch between installed Java versions
env choose java
```

### Update Repository

```bash
# Update the repository
# 更新仓库配置
python scripts/update_repository.py

# 设置定时任务
python scripts/update_repository.py --setup-cron
```

## Configuration

The tool stores its configuration in `$HOME/.env.config.json`. This includes:
- Installation directory
- Installed versions for each environment
- Current active versions
- Environment-specific settings

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
