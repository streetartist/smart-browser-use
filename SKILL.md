---
name: smart-browser-use
displayName: Smart Browser Use
summary: Use smart-browser-use MCP tools to operate live browser pages with inspection, stable interaction strategies, and verification loops.
primaryEnv: mcp
os:
  - windows
  - darwin
  - linux
requires:
  bins:
    - chrome
---

# Smart Browser Use

Use this skill when an agent controls a real browser through the `smart-browser-use` MCP server. It is for browser tasks where the page is alive: dynamic apps, logged-in sessions, multi-step workflows, forms, questionnaires, dashboards, and pages that change after each interaction.

The central habit is to treat MCP browser tools as a stateful control channel, not as a blind command executor. Build a model of the current page, act through the layer the page actually responds to, and verify the resulting state before moving on.

## MCP Usage Model

Use MCP tools as the primary interface. Do not add a parallel command shell, polling file, or project-specific REPL when the same action can be represented as an MCP tool.

Keep server startup and browser startup separate. Starting the MCP server should not immediately open Chrome. The browser should open lazily when a browser tool is called, and closing the browser should release the session so the next open starts fresh.

Expose browser capability as composable tools: lifecycle, navigation, inspection, interaction, state, debugging, and tabs. A good browser MCP server should let the agent inspect first, interact second, and verify afterward.

Prefer normal MCP interactions for ordinary pages:

- `browser_open`, `browser_close`, and tab tools for lifecycle.
- navigation and wait tools for page movement.
- snapshot, extraction, screenshot, and evaluate tools for inspection.
- click, fill, select, hover, key, and scroll tools for interaction.
- cookie, localStorage, console, and network tools for state and debugging.

Use `browser_evaluate` as a precision instrument, not as the default for everything. It is appropriate for inspecting hidden state, custom widgets, framework-managed controls, and dynamic pages where ordinary click/fill tools cannot expose enough information.

## Principles

Start from the user's current page when possible. If the user has already opened the target page, inspect that context instead of opening a new page or navigating away. Confirm the active tab with MCP tab tools before acting.

Prefer state over appearance. Visual structure and accessible text are useful for orientation, but hidden inputs, generated IDs, framework state, and event handlers often determine whether an action actually worked.

Use the simplest reliable interaction. A normal MCP click or fill is best when it works. For custom controls, target the real control or the visible proxy that owns the event handler. Avoid long keyboard simulations unless the page genuinely requires them.

Work in short verified phases. After selecting options, filling text, navigating, or submitting, read the state back through MCP inspection tools. A successful tool call is not the same thing as a successful page action.

Separate filling from submission. Completing fields prepares the page; submitting changes the outside world. Verify required state before submitting, and inspect the post-submit result afterward.

## Workflow

1. Establish context.
   - Identify the active page, current URL, title, and whether the user has already positioned the browser.
   - Avoid extra tabs unless they are part of the task.
   - Use lifecycle tools deliberately: open only when needed, close when done, and verify tab state after restarts.

2. Build a page model.
   - Identify major regions, dialogs, forms, repeated items, and navigation controls.
   - For forms, map question groups, labels, real controls, values, and submit targets.
   - Combine snapshot-style inspection with JavaScript state inspection when the page is dynamic.

3. Choose an interaction path.
   - Use visible controls for ordinary pages.
   - Use stable selectors, IDs, names, or state queries for dynamic pages.
   - Use direct value setting only as a fallback when normal input is too slow or ignored.
   - Keep direct JavaScript actions scoped and verifiable.

4. Execute conservatively.
   - Prefer sequential actions when each click can mutate the page.
   - Batch only when the page structure is stable and the targets are known.
   - Re-check state after each meaningful batch.

5. Finish with confirmation.
   - Validate required selections and text fields.
   - Submit only when the user asked for completion or explicitly approved it.
   - Report whether the result completed, failed validation, or requires human verification.

## Heuristics

Element indexes are temporary. They are useful for first discovery, but can become stale after rerenders, validation updates, modal openings, or dynamic section expansion.

Visible labels describe meaning; form state records truth. For forms, labels tell you what the answer means, while checked inputs, values, names, and framework state tell you what the page will submit.

The smallest actionable element is usually the right target. Avoid clicking a large container just because it contains the desired text. Find the button, link, input, label, or proxy element that owns the action.

Fast input should change value and notify listeners. Robust filling usually means focusing the element, setting its value through the native property when needed, and dispatching input/change events.

Return summaries, not live objects. When inspecting page state programmatically, return plain JSON-like data instead of DOM nodes, events, functions, or cyclic objects.

Tool schemas should stay MCP-compatible. Avoid top-level `oneOf`, `anyOf`, `allOf`, `enum`, or non-object parameter schemas when defining tools for clients that require strict JSON Schema compatibility.

MCP tool names should be capability-oriented and stable. Prefer names like `browser_open`, `browser_snapshot`, `browser_input_fill`, and `browser_get_local_storage` over workflow-specific commands such as `fill_questionnaire_site_x`.

## Experience Cases

### Build A Compact Control Map

For complex forms or dashboards, first collect a concise map of containers and controls. The goal is to learn how the page records state, not to hard-code one website.

```js
Array.from(document.querySelectorAll('form, fieldset, .field, [role="group"]')).map((group, i) => ({
  index: i,
  id: group.id,
  text: group.innerText?.slice(0, 200),
  controls: Array.from(group.querySelectorAll('input, textarea, select, button, a')).map(el => ({
    tag: el.tagName,
    id: el.id,
    name: el.name,
    type: el.type,
    value: el.value,
    text: (el.innerText || el.getAttribute('aria-label') || '').trim().slice(0, 120),
    className: el.className
  }))
}))
```

### Custom Choices Often Have Proxy Elements

Some pages hide real radio or checkbox inputs and attach behavior to a nearby visual element. Click the proxy when it exists, then verify the hidden input state.

```js
const input = document.getElementById('option_id');
const proxy = input?.parentElement?.querySelector('a, label, span, div');
proxy?.click();
```

### Verify Form State From Controls

After interaction, read the state from controls rather than from styling.

```js
(() => {
  const state = {};
  for (const input of document.querySelectorAll('input, textarea, select')) {
    const key = input.name || input.id;
    if (!key) continue;
    if (input.type === 'radio' || input.type === 'checkbox') {
      if (!state[key]) state[key] = [];
      if (input.checked) state[key].push(input.value);
    } else {
      state[key] = input.value;
    }
  }
  return state;
})()
```

### Reliable Text Fill Fallback

If ordinary filling is ignored, set the native value and dispatch the events most front-end frameworks listen for.

```js
(() => {
  const el = document.querySelector('#field');
  const value = 'Example text';
  el.focus();
  const proto = el.tagName === 'TEXTAREA'
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  const descriptor = Object.getOwnPropertyDescriptor(proto, 'value');
  if (descriptor?.set) descriptor.set.call(el, value);
  else el.value = value;
  el.dispatchEvent(new InputEvent('input', {
    bubbles: true,
    cancelable: true,
    inputType: 'insertReplacementText',
    data: value
  }));
  el.dispatchEvent(new Event('change', { bubbles: true }));
  return el.value;
})()
```

### Find The Actual Submit Target

Inspect small actionable candidates before clicking. The correct target is usually a button-like element, not the parent region containing the whole form.

```js
Array.from(document.querySelectorAll('button, input[type="submit"], input[type="button"], [role="button"], .submitbtn'))
  .map(el => ({
    tag: el.tagName,
    id: el.id,
    className: el.className,
    text: (el.innerText || el.value || '').trim(),
    visible: !!(el.offsetWidth || el.offsetHeight || el.getClientRects().length)
  }))
```

## Safety

Do not submit sensitive forms, purchases, financial operations, account changes, legal documents, medical records, job or school applications, or irreversible workflows unless the user explicitly authorizes the final submission.

For ordinary test forms and low-risk questionnaires, completing the task is acceptable when the user clearly asks for completion. Still verify before submitting and report any captcha, slider, or manual confirmation requirement.
