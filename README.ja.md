# Zed Unity Debugger

実行中の Unity Editor プロセスにアタッチして C# スクリプトをデバッグできる [Zed](https://zed.dev) 拡張機能です。

## 前提条件

**必須: Visual Studio Tools for Unity**（vstuc）VS Code 拡張 — [VS Code Marketplace からインストール](https://marketplace.visualstudio.com/items?itemName=VisualStudioToolsForUnity.vstuc)

この拡張機能は、vstuc のローカルインストールから `UnityDebugAdapter.dll` を検索します。DLL はこの拡張機能に同梱されていません。vstuc を自分でインストールし、[Microsoft のライセンス条項](https://marketplace.visualstudio.com/items/VisualStudioToolsForUnity.vstuc/license)に従う必要があります。

> **注意:** vstuc のライセンスは Visual Studio Code および関連する Microsoft 製品での使用を前提としています。Zed で `UnityDebugAdapter.dll` を使用することはその条項の範囲外です。ライセンスへの準拠はご自身の責任で判断してください。

その他の要件:
- [.NET 9](https://dotnet.microsoft.com/download)（UnityDebugAdapter.dll の実行に必要）
- **Editor Attaching** が有効な Unity Editor（Edit → Preferences → External Tools）

## セットアップ

### 1. Unity プロジェクトの設定

Unity プロジェクトのルートに `.zed/debug.json` を作成します:

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

### 2. 拡張機能のインストール

Zed で `Ctrl+Shift+X` → "Unity Debugger" を検索 → インストール

ソースからの Dev Extension としてインストールする場合は [コントリビューション](#コントリビューション) を参照してください。

### 3. DLL パスの設定（自動検出に失敗する場合）

`UnityDebugAdapter.dll` が自動検出されない場合は、Zed の設定（`Ctrl+,`）に以下を追加します:

```json
{
  "dap": {
    "Unity": {
      "binary": "C:\\Users\\<あなたのユーザー名>\\.vscode\\extensions\\visualstudiotoolsforunity.vstuc-<バージョン>\\bin\\UnityDebugAdapter.dll"
    }
  }
}
```

## 使い方

1. Unity Editor でプロジェクトを開いた状態にしておく
2. 同じプロジェクトフォルダを Zed で開く
3. Zed のデバッグパネルを開く
4. **"Attach to Unity Editor"** を選択してデバッグ開始
5. C# スクリプトにブレークポイントを設定 — Unity がそこに達すると Zed で停止する

## 仕組み

```
Zed ──DAP──▶ dotnet UnityDebugAdapter.dll ──TCP──▶ Unity Editor (ポート: 56000 + PID%1000)
```

この拡張機能は以下の手順で動作します:

1. `Library/EditorInstance.json` を読み込み、実行中の Unity Editor のプロセス ID を取得
2. デバッグポートを計算: `56000 + (processId % 1000)`
3. `dotnet` 経由で `UnityDebugAdapter.dll` を起動し、接続先エンドポイントを渡す
4. Zed が Debug Adapter Protocol（DAP）を通じてアダプターと通信

## トラブルシューティング

| エラー | 対処法 |
|--------|--------|
| `Unity Editor is not running` | まず Unity Editor でプロジェクトを開く |
| `Unity debug adapter not found` | vstuc をインストールするか、Zed 設定で `dap.Unity.binary` を指定する |
| `dotnet not found` | [.NET 9](https://dotnet.microsoft.com/download) をインストールする |
| スピナーが止まらない | Unity の Preferences で "Editor Attaching" が有効になっているか確認する |

## コントリビューション

```bash
git clone https://github.com/waless-seel/ZedUnityDebugger
cd ZedUnityDebugger

# ビルド
cargo build --target wasm32-wasip1 --release

# Zed に Dev Extension としてインストール
# Ctrl+Shift+X → Install Dev Extension → このディレクトリを選択
```

## ライセンス

MIT — [LICENSE](LICENSE) を参照

この拡張機能は Microsoft のソフトウェアを同梱・配布しません。
`UnityDebugAdapter.dll` は Microsoft が所有し、[vstuc ライセンス](https://marketplace.visualstudio.com/items/VisualStudioToolsForUnity.vstuc/license)に基づいて提供されています。
