# douban

一个极其简单的豆瓣电影 / 剧集信息查询工具。双击即用，无需安装任何运行时环境。

## 使用方法

从 [Releases](https://github.com/mflix-team/douban/releases) 下载对应平台产物，解压后双击运行：

| 平台 | 产物 | 说明 |
| --- | --- | --- |
| macOS | `douban-macos.zip` → `douban.app` | 通用二进制，同时支持 Apple 芯片与 Intel Mac |
| Windows | `douban.exe` | 支持 Windows 10+ |

首次启动会自动打开浏览器并进入查询页面，在页面上输入豆瓣条目 ID（或完整链接）即可查询：

- 电影 / 剧集信息（标题、年份、评分、海报、导演、编剧、演员、简介、上映日期、片长、又名）
- 支持「解析视图 / JSON 视图」切换，选择会自动记忆
- 查询历史记录保存在本地
- 随机轮换 UA，降低被豆瓣拦截的概率；控制好频率就不会被拉黑

> macOS 产物未做代码签名，首次打开如被 Gatekeeper 拦截，请在「系统设置 → 隐私与安全性」或右键 → 打开 中放行一次。

## 开发运行

需要 Rust 工具链（跨平台）：

```bash
cargo run --release
```

## 构建与发布

GitHub Actions（`.github/workflows/build.yml`）会在 `master` 分支推送时分别在 macOS / Windows 上构建产物：

- macOS：交叉编译 `aarch64-apple-darwin` 与 `x86_64-apple-darwin`，`lipo` 合并为 universal 二进制并打包为 `.app`
- Windows：构建 `douban.exe`

产物以上传构建产物（Artifacts）保存；推送 `v*` 标签时自动创建 GitHub Release 并附带双平台产物。

本地构建：

```bash
cargo build --release
# 产物：target/release/douban（macOS 本机架构）/ douban.exe（Windows）
```

## License

[MIT](./LICENSE)
