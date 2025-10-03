# 贡献指南

感谢您对 Smart Bangumi qBittorrent Rule Generator 项目的关注！我们欢迎各种形式的贡献。

## 开发环境设置

### 前提条件
- Rust 1.70+
- Git
- 网络连接

### 本地开发

1. **Fork 仓库**
   ```bash
   git clone https://github.com/your-username/smart-bangumi-qb-rule-generator.git
   cd smart-bangumi-qb-rule-generator
   ```

2. **构建项目**
   ```bash
   cargo build
   cargo test
   ```

3. **运行测试**
   ```bash
   cargo test
   cargo test -- --nocapture  # 查看测试输出
   ```

## 贡献类型

### 🐛 报告 Bug
- 使用 GitHub Issues 报告 bug
- 提供详细的复现步骤
- 包括错误信息和系统环境

### 💡 功能请求
- 描述新功能的使用场景
- 说明为什么这个功能很重要
- 如果可能，提供实现思路

### 🔧 代码贡献
1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 代码规范

### Rust 代码风格
- 遵循 Rust 官方代码风格
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 为公共 API 添加文档注释

### 提交信息规范
- 使用英文提交信息
- 格式：`类型: 描述`
- 类型：`feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### 测试要求
- 为新功能添加单元测试
- 确保所有测试通过
- 测试覆盖率尽量提高

## 项目结构

```
src/
├── main.rs              # 主应用程序逻辑
├── ai/
│   └── object_matcher/  # AI 对象匹配系统
└── meta_providers/
    └── bangumi/         # Bangumi API 集成
```

## 扩展项目

### 添加新网站支持
1. 在 `SiteType` 枚举中添加新站点
2. 在 `main()` 的匹配语句中添加站点处理器
3. 实现站点特定的表格提取

### 添加新 AI 提供商
1. 在 `AiProvider` 枚举中添加新提供商
2. 添加 API 密钥处理逻辑
3. 实现提供商特定的 API 调用

## 发布流程

### 版本管理
- 使用语义化版本 (SemVer)
- 主版本号：不兼容的 API 修改
- 次版本号：向下兼容的功能性新增
- 修订号：向下兼容的问题修正

### 发布检查清单
- [ ] 所有测试通过
- [ ] 代码格式化完成
- [ ] 文档更新完成
- [ ] 版本号更新
- [ ] 发布说明编写

## 许可证

本项目采用 MIT 许可证。通过提交贡献，您同意您的贡献将在相同的许可证下发布。

## 联系方式

- GitHub Issues: 报告问题和功能请求
- Pull Requests: 提交代码贡献
- Discussions: 技术讨论和问题解答

感谢您的贡献！🎉