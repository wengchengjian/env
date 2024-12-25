# env - 开发环境管理器

一个用于高效管理多个开发环境的命令行工具。

## 功能特性

- 轻松安装和管理以下环境的不同版本：
  - Java (JDK)
  - Python (即将推出)
  - Node.js (即将推出)
  - Rust (即将推出)
  - Go (即将推出)
  - MySQL (即将推出)
  - Redis (即将推出)
- 自动配置环境变量
- 简单的版本切换功能
- 全局配置管理

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
# 安装指定环境
env dev java

# 安装所有支持的环境
env dev --all

# 切换版本
env choose java

# 配置安装目录
env config --dir "C:\Program Files\env"

# 刷新环境配置
env config --flush
```

### Java环境管理

```bash
# 安装指定版本的Java
env dev java -v 17.0.9

# 在已安装的Java版本间切换
env choose java
```

## 配置

工具的配置文件存储在 `$HOME/.env.config.json`，包含以下信息：
- 安装目录
- 每个环境的已安装版本
- 当前激活的版本
- 环境特定的设置

## 贡献

欢迎提交Pull Request来帮助改进这个项目！

## 许可证

本项目采用MIT许可证 - 详见LICENSE文件
