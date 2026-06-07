# GitHub Actions 自动构建说明

## 配置方法

1. 在 GitHub 创建仓库
2. 推送代码到 GitHub
3. GitHub Actions 会自动运行

## 工作流说明

### 1. build.yml - 自动构建
每次推送代码时自动构建所有平台：
- macOS (Intel x86_64)
- macOS (Apple Silicon ARM64)
- Linux (x86_64)
- Windows (x86_64)

### 2. test.yml - 自动测试
每次推送时运行测试：
- 单元测试
- 集成测试
- 代码格式化检查
- Clippy 静态分析

### 3. release.yml - 手动发布
手动触发发布流程：
1. 进入 Actions 页面
2. 选择 "Manual Release"
3. 点击 "Run workflow"
4. 输入版本号（如 v0.1.0）
5. 自动构建并创建 Release

## 使用方法

### 自动构建（每次推送）
代码推送到 main 分支后自动构建，产物在 Actions 页面下载。

### 手动发布
```
1. 进入 GitHub 仓库
2. 点击 Actions 标签
3. 选择 "Manual Release"
4. 点击 "Run workflow"
5. 输入版本号
6. 等待构建完成
7. 在 Releases 页面下载
```

### 打标签自动发布
```bash
git tag v0.1.0
git push origin v0.1.0
```
自动触发构建并创建 Release。
