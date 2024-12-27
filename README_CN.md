# env - 开发环境管理器

一个强大的命令行工具，用于高效管理多个开发环境。它可以帮助开发者轻松安装、管理和切换不同版本的开发工具和运行时环境。

## 功能特性

- 轻松安装和管理以下环境的不同版本：
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
- 自动配置环境变量
- 简单的版本切换功能
- 全局配置管理
- 交互式安装过程

## 安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/env.git
cd env

# 构建项目
cargo build --release

# 添加到系统PATH
# 可执行文件位于 target/release/env
```

## 使用方法

### 基本命令

```bash
# 安装开发环境（交互模式）
env dev

# 安装指定环境
env dev java
env dev python
env dev node

# 切换已安装环境的版本
env choose java
env choose python
env choose node

# 配置安装目录
env config --dir "C:\Program Files\env"

# 刷新环境配置
env config --flush

# 查看当前配置
env config
```

### 支持的环境

工具当前支持以下环境：
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

每个环境都可以使用相同的命令模式进行安装和管理：
```bash
env dev [环境名称]
env choose [环境名称]
```

## 配置

工具通过两个主要文件管理配置：

1. `$HOME/.env.config.json`：用户配置文件，包含：
   - 安装目录路径
   - 每个环境的已安装版本
   - 当前激活的版本
   - 环境特定的设置

2. `.env.config.default.json`：默认配置模板，定义：
   - 可用的环境列表
   - 下载源配置
   - 环境特定的配置

## 贡献

欢迎提交Pull Request来帮助改进这个项目！

## 许可证

本项目采用MIT许可证 - 详见 [LICENSE](LICENSE) 文件
