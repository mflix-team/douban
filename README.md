# douban

一个极其简单的豆瓣电影 / 剧集信息查询工具，双击即用、无需安装任何环境。

![douban](./douban.png)

## 使用方法

双击运行 `douban`（Windows 为 `douban.exe`），会自动打开浏览器，在页面上输入豆瓣条目 ID（或完整链接）即可查询：

- 电影 / 剧集信息（标题、年份、评分、海报、导演、编剧、演员、简介、上映日期、片长、又名）
- 支持「解析视图 / JSON 视图」切换，选择会自动记忆
- 查询历史记录保存在本地
- 随机轮换 UA，降低被豆瓣拦截的概率；控制好频率就不会被拉黑

## 开发运行

需要 Rust 工具链（cross-platform）：

```bash
cargo run --release
```

## 构建与发布

GitHub Actions（`.github/workflows/build.yml`）会在 `master` 分支推送时分别在 macOS / Windows 上构建对应平台的原生可执行文件，作为构建产物上传；推送 `v*` 标签时自动创建 GitHub Release 并附带双平台二进制。

本地构建：

```bash
cargo build --release
# 产物：target/release/douban（macOS）/ douban.exe（Windows）
```