# Bangumi Rules Builder

一个智能的 Rust 应用程序，通过网页抓取、AI 处理和 Bangumi API 集成，自动为动漫季度生成 qBittorrent RSS 下载规则。

## 功能特点

### 🎯 核心功能
- **智能网页抓取**：从 kansou.me 等网站提取动漫信息
- **AI 增强处理**：使用 AI API 清理标题并生成搜索关键词
- **Bangumi 集成**：通过 Bangumi API 搜索官方动漫信息
- **自动规则生成**：创建 qBittorrent RSS 下载规则
- **跨平台 GUI 编辑器**：Python tkinter 界面编辑生成的规则

### 🚀 技术特性
- **多语言支持**：日语、中文、英文标题处理
- **智能匹配**：基于权重评分的动漫匹配算法
- **批量处理**：AI API 批量处理以管理使用限制
- **缓存机制**：避免重复 API 调用
- **错误处理**：全面的错误处理和优雅降级

## 快速开始

### 系统要求
- Windows 10+ / Linux / macOS
- qBittorrent 4.4+
- Python 3.8+ (用于 GUI 编辑器)
- 网络连接 (用于 API 调用)
- DeepSeek API Key (用于 AI 处理)

### 安装

#### 使用预编译二进制
从 [Releases](https://github.com/thelastfantasy/Bangumi-Rules-Builder/releases) 页面下载对应平台的二进制文件。

### 配置

1. **设置 API 密钥**：

   **Windows (命令提示符):**
   ```cmd
   set DEEPSEEK_API_KEY=your_deepseek_api_key
   ```

   **Windows (PowerShell):**
   ```powershell
   $env:DEEPSEEK_API_KEY="your_deepseek_api_key"
   ```

   **Linux/macOS:**
   ```bash
   export DEEPSEEK_API_KEY="your_deepseek_api_key"
   ```

2. **编辑任务配置** (`tasks.json`)：
   ```json
   {
     "description": "2025年10月新番",
     "site": "kansou",
     "root_path": "E:\\Anime\\新番"
   }
   ```

3. **运行程序**：

   **Windows:**
   ```cmd
   bangumi-rules-builder.exe
   ```

   **Linux:**
   ```bash
   ./bangumi-rules-builder
   ```

   **macOS:**
   ```bash
   ./bangumi-rules-builder
   ```

### 使用 GUI 编辑器

```bash
# 使用 Python 运行
python qb_rule_editor.py

# 或使用启动脚本 (Windows)
run_editor.bat
run_editor.ps1
```

## 项目结构

```
src/
├── main.rs              # 主应用程序逻辑
├── ai/
│   └── object_matcher/  # AI 对象匹配系统
└── meta_providers/
    └── bangumi/         # Bangumi API 集成

# 配置文件
tasks.json               # 处理配置
.gitignore              # Git 忽略规则

# 工具脚本
qb_rule_editor.py       # Python GUI 编辑器
run_editor.bat          # Windows 启动脚本
run_editor.ps1          # PowerShell 启动脚本
```

## 工作流程

### 核心处理流程

1. **配置加载** (`main.rs`)
   - 从 `tasks.json` 读取任务配置
   - 解析站点类型和描述

2. **网站抓取** (`sites/kansou.rs`)
   - 从 kansou.me 获取HTML页面
   - 提取包含动漫信息的表格
   - 解析作品标题和播出日期

3. **AI处理** (`ai/deepseek/mod.rs`)
   - 使用DeepSeek API智能选择正确的表格
   - 清理和标准化作品标题
   - 生成多语言搜索关键词

4. **Bangumi集成** (`meta_providers/bangumi/mod.rs`)
   - 通过Bangumi API搜索官方信息
   - **智能AI匹配**: 使用DeepSeek AI进行语义匹配，考虑标题相似性、放映时间、关键词匹配
   - 提取中文名称和别名

5. **规则生成** (`rules/q_bittorrent/mod.rs`)
   - 创建qBittorrent RSS下载规则
   - 设置下载路径和分类
   - 生成智能过滤模式

### 关键改进

- **模块化架构**: 清晰的职责分离，易于扩展新站点和AI提供商
- **AI智能匹配**: 使用DeepSeek AI进行语义匹配，替代传统的score-based算法
  - 考虑标题语义相似性（包括特殊符号、季度表示差异）
  - 放映时间的接近程度
  - 关键词与候选作品标题/别名的匹配度
  - 是否为同一作品的不同季度
- **批量处理**: AI API批量处理，优化性能和成本
- **错误处理**: 完善的错误处理和统计跟踪

## 开发

### 构建和测试

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 检查代码
cargo check
```

### 依赖项

- **serde/serde_json**: JSON序列化
- **reqwest**: HTTP客户端
- **scraper**: HTML解析
- **tokio**: 异步运行时
- **regex**: 正则表达式
- **chrono**: 日期时间处理

### 扩展项目

#### 添加新站点
1. 在 `models.rs` 的 `SiteType` 枚举中添加新站点
2. 在 `sites/` 目录下创建新的站点模块
3. 在 `main.rs` 的匹配语句中添加站点处理逻辑
4. 实现站点特定的表格提取和作品解析

#### 添加新AI提供商
1. 在 `models.rs` 的 `AiProvider` 枚举中添加新提供商
2. 在 `ai/` 目录下创建新的提供商模块
3. 在 `ai/mod.rs` 中注册新的提供商
4. 实现提供商特定的API调用逻辑

#### 添加新元数据提供者
1. 在 `meta_providers/` 目录下创建新的提供者模块
2. 实现提供者特定的搜索和匹配逻辑
3. 在 `meta_providers/mod.rs` 中注册新的提供者

## 许可证

MIT License

## 贡献

欢迎提交Issue和Pull Request！

### 开发指南

- 遵循Rust命名约定和代码风格
- 为公共函数和结构体添加文档注释
- 使用 `Result<_, Box<dyn std::error::Error>>` 进行错误处理
- 为外部API兼容性使用 `#[serde(rename)]`
- 详细的开发文档请参考 `CLAUDE.md`