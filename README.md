# Zed Unity Debugger

A [Zed](https://zed.dev) extension that lets you debug Unity C# scripts by attaching to a running Unity Editor process.

## Prerequisites

**Required: Visual Studio Tools for Unity** (vstuc) VS Code extension — [install from VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=VisualStudioToolsForUnity.vstuc)

This extension locates `UnityDebugAdapter.dll` from your local vstuc installation. The DLL is **not bundled** with this extension; you must install vstuc yourself and comply with [Microsoft's license terms](https://marketplace.visualstudio.com/items/VisualStudioToolsForUnity.vstuc/license).

> **Note:** vstuc's license restricts use to Visual Studio Code and related Microsoft products. Using `UnityDebugAdapter.dll` with Zed is outside those terms. You are responsible for compliance with the vstuc license.

Other requirements:
- [.NET 9](https://dotnet.microsoft.com/download) (to run UnityDebugAdapter.dll)
- Unity Editor with **Editor Attaching** enabled (Edit → Preferences → External Tools)

## Setup

### 1. Configure your Unity project

Create `.zed/debug.json` in your Unity project root:

```json
[
  {
    "adapter": "Unity",
    "label": "Attach to Unity Editor",
    "request": "attach",
    "projectPath": "${ZED_WORKTREE_ROOT}"
  }
]
```

### 2. Install this extension

In Zed: `Ctrl+Shift+X` → search "Unity Debugger" → Install

Or install as a dev extension from source (see [Contributing](#contributing)).

### 3. DLL path (if auto-detection fails)

If `UnityDebugAdapter.dll` is not found automatically, add to Zed settings (`Ctrl+,`):

```json
{
  "dap": {
    "Unity": {
      "binary": "C:\\Users\\<you>\\.vscode\\extensions\\visualstudiotoolsforunity.vstuc-<version>\\bin\\UnityDebugAdapter.dll"
    }
  }
}
```

## Usage

1. Open your Unity project in Unity Editor (keep it running)
2. Open the same project folder in Zed
3. Open Zed's Debug panel
4. Select **"Attach to Unity Editor"** and start debugging
5. Set breakpoints in your C# scripts — Zed will pause at them when Unity hits them

## How it works

```
Zed ──DAP──▶ dotnet UnityDebugAdapter.dll ──TCP──▶ Unity Editor (port 56000 + PID%1000)
```

This extension:
1. Reads `Library/EditorInstance.json` to find the running Unity Editor's process ID
2. Calculates the debug port: `56000 + (processId % 1000)`
3. Launches `UnityDebugAdapter.dll` via `dotnet`, passing the endpoint
4. Zed communicates with the adapter over the Debug Adapter Protocol (DAP)

## Troubleshooting

| Error | Fix |
|-------|-----|
| `Unity Editor is not running` | Open your project in Unity Editor first |
| `Unity debug adapter not found` | Install vstuc or set `dap.Unity.binary` in Zed settings |
| `dotnet not found` | Install [.NET 9](https://dotnet.microsoft.com/download) |
| Spinner keeps going | Verify "Editor Attaching" is enabled in Unity Preferences |

## Contributing

```bash
git clone https://github.com/seless/zed-unity-debugger
cd zed-unity-debugger

# Build
cargo build --target wasm32-wasip1 --release

# Install as dev extension in Zed
# Ctrl+Shift+X → Install Dev Extension → select this directory
```

## License

MIT — see [LICENSE](LICENSE)

This extension does not include or distribute any Microsoft software.
`UnityDebugAdapter.dll` is owned by Microsoft and governed by the
[vstuc license](https://marketplace.visualstudio.com/items/VisualStudioToolsForUnity.vstuc/license).
