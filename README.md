# smart-browser-use

[中文文档](README.zh-CN.md)

`smart-browser-use` is a Rust browser automation library and MCP server for controlling Chrome through the Chrome DevTools Protocol (CDP). It is designed for AI agents and automation systems that need to work with real, stateful web pages: dynamic applications, forms, dashboards, logged-in sessions, screenshots, DOM extraction, cookies, localStorage, and browser tabs.

The project focuses on practical browser control rather than a custom command shell. Start the server, connect with an MCP-capable client, and call browser tools directly.

Repository: https://github.com/streetartist/smart-broswer-use

Reworked from: https://github.com/rknoche6/fast-browser-use

## What It Provides

- Real Chrome/Chromium control through CDP.
- A standard MCP server for AI-driven browser automation.
- Lazy browser startup: starting the server does not immediately launch Chrome.
- Headed and headless modes.
- Page navigation, back/forward, waiting, scrolling, screenshots, and page snapshots.
- Element interaction by CSS selector or indexed page snapshot.
- Text input, click, hover, select, keyboard, and tab management tools.
- Cookie and localStorage access.
- Console log and network error inspection.
- Sitemap and page structure analysis.
- Support for launching a new browser or connecting to an existing Chrome DevTools WebSocket endpoint.

## Status

This project is actively evolving. The current direction is:

- Use MCP as the main integration surface.
- Keep browser startup explicit and lazy.
- Prefer stable browser state inspection over brittle command scripts.
- Improve form and dynamic page handling based on real-world agent workflows.

## Requirements

- Rust toolchain.
- Chrome, Chromium, or Microsoft Edge installed.
- An MCP-capable client if you want to use the server integration.

## Installation

Clone the repository:

```bash
git clone https://github.com/streetartist/smart-broswer-use.git
cd smart-broswer-use
```

Build the MCP server:

```bash
cargo build --features mcp-server --bin mcp-server
```

Run checks:

```bash
cargo check --features mcp-server --bin mcp-server
```

## Running The MCP Server

Start the server in stdio mode:

```bash
cargo run --features mcp-server --bin mcp-server
```

Start with a visible browser when the first browser tool is called:

```bash
cargo run --features mcp-server --bin mcp-server -- --headed
```

Specify a Chrome executable:

```bash
cargo run --features mcp-server --bin mcp-server -- --headed --executable-path "C:\Program Files\Google\Chrome\Application\chrome.exe"
```

Use a persistent Chrome profile:

```bash
cargo run --features mcp-server --bin mcp-server -- --headed --user-data-dir .\chrome-profile
```

The server starts without opening Chrome. Chrome is launched lazily when a browser tool such as `browser_open` is called.

## Connecting To An Existing Browser

Start Chrome manually with remote debugging:

```bash
chrome --remote-debugging-port=9222 --user-data-dir=/tmp/smart-browser-use-profile
```

Read the browser WebSocket URL:

```text
http://127.0.0.1:9222/json/version
```

Then start the MCP server with that endpoint:

```bash
mcp-server --ws-endpoint ws://127.0.0.1:9222/devtools/browser/...
```

This mode is useful when the user manually opens a page or logs in first, and the agent should continue from that browser state.

## MCP Tools

The server exposes browser tools including:

### Browser Lifecycle

- `browser_open`: Open or connect to the browser lazily.
- `browser_close`: Close the browser session and allow the next open to start fresh.
- `browser_tab_list`: List tabs.
- `browser_new_tab`: Open a new tab.
- `browser_switch_tab`: Switch tabs.
- `browser_close_tab`: Close the active tab.

### Navigation

- `browser_navigate`: Navigate to a URL.
- `browser_go_back`: Go back.
- `browser_go_forward`: Go forward.
- `browser_wait`: Wait for an element.
- `browser_scroll`: Scroll the page.

### Page Inspection

- `browser_snapshot`: Return an indexed page snapshot for agent use.
- `browser_extract`: Extract text or HTML.
- `browser_get_markdown`: Convert page content to Markdown.
- `browser_read_links`: Read links from the current page.
- `browser_screenshot`: Capture a screenshot.
- `browser_annotate`: Capture an annotated screenshot.
- `browser_evaluate`: Execute JavaScript and return serializable results.

### Interaction

- `browser_click`: Click by CSS selector or snapshot index.
- `browser_input_fill`: Fill text fields quickly and dispatch input/change events.
- `browser_select`: Select an option.
- `browser_hover`: Hover an element.
- `browser_press_key`: Press a keyboard key.

### State And Debugging

- `browser_get_cookies`: Read cookies.
- `browser_set_cookies`: Set cookies.
- `browser_get_local_storage`: Read localStorage.
- `browser_set_local_storage`: Set localStorage.
- `browser_remove_local_storage`: Remove one localStorage item.
- `browser_clear_local_storage`: Clear localStorage.
- `browser_get_console_logs`: Read captured console logs.
- `browser_get_network_errors`: Read captured network errors.
- `browser_sitemap`: Analyze sitemap URLs and page structure.

## Form And Dynamic Page Guidance

Dynamic forms often use hidden inputs, visual proxy elements, and framework-managed state. A reliable automation flow is:

1. Inspect the page structure with a snapshot.
2. Use JavaScript evaluation to map real inputs, names, IDs, values, and checked states.
3. Interact with visible controls when possible.
4. For custom controls, click the proxy element tied to the hidden input.
5. Verify selected values from the underlying form state.
6. Fill text fields with `browser_input_fill`, or use direct value setting with input/change events when necessary.
7. Locate the real submit element, not a large container that merely contains submit text.
8. After submission, check for validation messages, success pages, or human verification.

## Rust Library Usage

The package name is `smart-browser-use`, while the Rust library crate remains `browser_use` for compatibility.

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

## Development

Format:

```bash
cargo fmt
```

Check:

```bash
cargo check --features mcp-server --bin mcp-server
```

Build:

```bash
cargo build --features mcp-server --bin mcp-server
```

## Notes

- The repository name intentionally follows the provided URL: `smart-broswer-use`.
- The package and skill name are `smart-browser-use`.
- The library crate name is currently `browser_use` to avoid breaking existing Rust imports.

## License

GPL-v3.0
