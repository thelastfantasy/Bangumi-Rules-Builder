# Bangumi Rules Builder

一个智能的动漫下载规则生成器，通过网页抓取、AI处理和Bangumi API集成，自动为qBittorrent生成RSS下载规则。

## 功能特性

- 🕷️ **网页抓取**: 自动从kansou.me等网站提取动漫信息
- 🤖 **AI处理**: 使用DeepSeek API清理标题并生成搜索关键词
- 📚 **Bangumi集成**: 通过Bangumi API获取官方动漫信息
- 📥 **规则生成**: 为qBittorrent创建智能RSS下载规则
- 🌍 **多语言支持**: 支持日文、中文、英文标题变体
- 📊 **统计报告**: 提供详细的处理统计和API使用情况
- 🔧 **模块化架构**: 清晰的模块分离，易于扩展和维护

## 快速开始

### 环境要求

- Rust 1.70+
- DeepSeek API密钥

### 安装

```bash
# 克隆项目
git clone <repository-url>
cd smart_bangumi_qb_rule_generator

# 构建项目
cargo build --release
```

### 配置

1. 设置环境变量：
```bash
export DEEPSEEK_API_KEY="your_deepseek_api_key_here"
```

2. 编辑 `tasks.json`：
```json
{
  "description": "2025年10月新番",
  "site": "kansou",
  "root_path": "E:\\Anime\\新番"
}
```

#### tasks.json 配置说明

- **description**: 描述文本，用于让DeepSeek AI识别对应的动漫季。格式灵活，只要能准确描述目标动漫季即可，例如：
  - "2025年10月新番"
  - "2025年秋季动画"
  - "2025年10月动漫列表"
  - "2025年10月番剧"

- **site**: 目前仅支持 "kansou"（kansou.me网站），未来计划支持更多站点：
  - **kansou** (当前支持) - 从kansou.me抓取动漫信息
  - **myanimelist** (计划中) - 从MyAnimeList获取数据
  - **modelscope** (计划中) - 使用ModelScope API

- **root_path**: 下载文件的根目录路径，程序会自动创建季节和作品名称的子文件夹

### 运行

```bash
cargo run
```

## 输出文件

- `qb_download_rules.json`: 生成的qBittorrent RSS规则
- `bangumi_results.json`: 缓存的Bangumi API结果

## 项目结构

```
src/
├── main.rs              # 主应用程序逻辑和入口点
├── models.rs            # 数据模型定义
├── sites/               # 网站抓取模块
│   ├── mod.rs
│   └── kansou.rs       # kansou.me网站处理
├── ai/                  # AI处理模块
│   ├── mod.rs
│   └── deepseek/       # DeepSeek API实现
│       └── mod.rs
├── meta_providers/      # 元数据提供者
│   ├── mod.rs
│   └── bangumi/        # Bangumi API集成
│       └── mod.rs
├── rules/               # 规则生成模块
│   ├── mod.rs
│   └── q_bittorrent/   # qBittorrent规则生成
│       └── mod.rs
└── utils.rs             # 工具函数

Cargo.toml              # 项目依赖配置
tasks.json              # 处理配置
qb_download_rules.json  # 生成的下载规则
bangumi_results.json    # 缓存的Bangumi结果
CLAUDE.md              # 详细开发文档
README.md              # 项目说明
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
   - 使用加权评分系统匹配作品
   - 提取中文名称和别名

5. **规则生成** (`rules/q_bittorrent/mod.rs`)
   - 创建qBittorrent RSS下载规则
   - 设置下载路径和分类
   - 生成智能过滤模式

### 关键改进

- **模块化架构**: 清晰的职责分离，易于扩展新站点和AI提供商
- **智能匹配**: 改进的Bangumi匹配算法，提高准确性
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