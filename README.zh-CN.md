# smart-browser-use

[English README](README.md)

`smart-browser-use` 是一个基于 Rust 的浏览器自动化库和 MCP 服务端，通过 Chrome DevTools Protocol（CDP）控制真实的 Chrome/Chromium 浏览器。它面向需要操作真实网页状态的 AI Agent 和自动化系统，例如动态应用、表单、问卷、后台页面、登录态页面、截图、DOM 提取、Cookie、localStorage 和多标签页管理。

项目重点不是自定义命令行交互壳，而是提供标准化的浏览器控制能力。启动服务后，由支持 MCP 的客户端直接调用浏览器工具。

仓库地址：https://github.com/streetartist/smart-broswer-use

## 能力概览

- 通过 CDP 控制真实 Chrome/Chromium。
- 内置标准 MCP 服务端，便于 AI Agent 调用。
- 懒启动浏览器：启动服务端时不会立刻打开 Chrome。
- 支持有界面和无界面模式。
- 支持导航、前进后退、等待、滚动、截图和页面快照。
- 支持通过 CSS selector 或页面快照索引定位元素。
- 支持点击、输入、悬停、下拉选择、按键和标签页管理。
- 支持 Cookie 和 localStorage 读写。
- 支持控制台日志和网络错误检查。
- 支持 sitemap 和页面结构分析。
- 可启动新浏览器，也可连接已有 Chrome DevTools WebSocket 端点。

## 当前方向

这个项目仍在持续迭代。当前设计方向是：

- 以 MCP 作为主要集成接口。
- 服务启动和浏览器启动分离，浏览器按需打开。
- 用稳定的页面状态检查替代脆弱的脚本命令。
- 基于真实 Agent 工作流持续增强表单和动态页面处理能力。

## 环境要求

- Rust 工具链。
- 已安装 Chrome、Chromium 或 Microsoft Edge。
- 如果使用服务端集成，需要一个支持 MCP 的客户端。

## 安装与构建

克隆仓库：

```bash
git clone https://github.com/streetartist/smart-broswer-use.git
cd smart-broswer-use
```

构建 MCP 服务端：

```bash
cargo build --features mcp-server --bin mcp-server
```

运行检查：

```bash
cargo check --features mcp-server --bin mcp-server
```

## 运行 MCP 服务端

以 stdio 模式启动：

```bash
cargo run --features mcp-server --bin mcp-server
```

在首次调用浏览器工具时打开可见 Chrome：

```bash
cargo run --features mcp-server --bin mcp-server -- --headed
```

指定 Chrome 路径：

```bash
cargo run --features mcp-server --bin mcp-server -- --headed --executable-path "C:\Program Files\Google\Chrome\Application\chrome.exe"
```

使用持久化浏览器用户目录：

```bash
cargo run --features mcp-server --bin mcp-server -- --headed --user-data-dir .\chrome-profile
```

服务端启动时不会立刻打开 Chrome。只有调用 `browser_open` 等浏览器工具时，才会按需启动浏览器。

## 连接用户已打开的浏览器

先手动启动带远程调试端口的 Chrome：

```bash
chrome --remote-debugging-port=9222 --user-data-dir=/tmp/smart-browser-use-profile
```

读取浏览器 WebSocket 地址：

```text
http://127.0.0.1:9222/json/version
```

然后用该端点启动服务端：

```bash
mcp-server --ws-endpoint ws://127.0.0.1:9222/devtools/browser/...
```

这个模式适合用户先手动打开页面、登录账号或定位到某个页面，然后让 Agent 接着操作当前浏览器状态。

## MCP 工具

服务端暴露的工具包括：

### 浏览器生命周期

- `browser_open`：按需打开或连接浏览器。
- `browser_close`：关闭浏览器会话，下次打开时重新启动。
- `browser_tab_list`：列出标签页。
- `browser_new_tab`：打开新标签页。
- `browser_switch_tab`：切换标签页。
- `browser_close_tab`：关闭当前标签页。

### 导航

- `browser_navigate`：访问 URL。
- `browser_go_back`：后退。
- `browser_go_forward`：前进。
- `browser_wait`：等待元素出现。
- `browser_scroll`：滚动页面。

### 页面检查

- `browser_snapshot`：返回带索引的页面快照，便于 Agent 理解页面。
- `browser_extract`：提取文本或 HTML。
- `browser_get_markdown`：将页面内容转为 Markdown。
- `browser_read_links`：读取当前页面链接。
- `browser_screenshot`：截图。
- `browser_annotate`：生成带元素标注的截图。
- `browser_evaluate`：执行 JavaScript 并返回可序列化结果。

### 交互

- `browser_click`：通过 CSS selector 或快照索引点击元素。
- `browser_input_fill`：快速填充文本字段，并触发 input/change 事件。
- `browser_select`：选择下拉框选项。
- `browser_hover`：悬停元素。
- `browser_press_key`：发送键盘按键。

### 状态与调试

- `browser_get_cookies`：读取 Cookie。
- `browser_set_cookies`：设置 Cookie。
- `browser_get_local_storage`：读取 localStorage。
- `browser_set_local_storage`：设置 localStorage。
- `browser_remove_local_storage`：删除单个 localStorage 项。
- `browser_clear_local_storage`：清空 localStorage。
- `browser_get_console_logs`：读取捕获到的控制台日志。
- `browser_get_network_errors`：读取网络错误。
- `browser_sitemap`：分析 sitemap URL 和页面结构。

## 表单与动态页面经验

动态表单常见隐藏输入、自定义可视控件和前端框架状态。更可靠的自动化流程是：

1. 先用页面快照理解整体结构。
2. 用 JavaScript 检查真实 input、name、id、value 和 checked 状态。
3. 普通控件优先走正常点击和填充。
4. 自定义控件需要找到隐藏 input 旁边真正绑定事件的可视代理元素。
5. 选择后从底层表单状态读取并验证。
6. 文本字段优先使用 `browser_input_fill`，必要时直接设置 value 并触发 input/change 事件。
7. 提交时定位真正的提交按钮，不要点到包含“提交”文字的大容器。
8. 提交后检查校验错误、成功页或人工验证。

## Rust 库用法

包名是 `smart-browser-use`，Rust 库 crate 名暂时保留为 `browser_use`，以兼容已有代码。

```rust
use browser_use::browser::BrowserSession;

fn main() -> browser_use::error::Result<()> {
    let session = BrowserSession::launch(Default::default())?;
    session.navigate("https://example.com")?;

    let dom = session.extract_dom()?;
    println!("{:#?}", dom.root);

    Ok(())
}
```

## 开发

格式化：

```bash
cargo fmt
```

检查：

```bash
cargo check --features mcp-server --bin mcp-server
```

构建：

```bash
cargo build --features mcp-server --bin mcp-server
```

## 说明

- 仓库名按提供的 URL 使用：`smart-broswer-use`。
- 包名和技能名为：`smart-browser-use`。
- Rust 库 crate 名当前保留为：`browser_use`。

## 许可证

GPL-v3.0
