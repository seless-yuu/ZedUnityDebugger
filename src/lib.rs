use serde::Deserialize;
use serde_json::Value;
use zed_extension_api::{
    self as zed, DebugAdapterBinary, DebugConfig, DebugRequest, DebugScenario,
    DebugTaskDefinition, StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest,
    Worktree,
};

/// Unity 固有のデバッグ設定 (debug.json の adapter-specific フィールド)
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UnityDebugConfig {
    project_path: Option<String>,
    end_point: Option<String>,
    log_file: Option<String>,
}

struct UnityDebuggerExtension;

impl zed::Extension for UnityDebuggerExtension {
    fn new() -> Self {
        Self
    }

    /// DAP サーバー（dotnet UnityDebugAdapter.dll）の起動コマンドを返す
    fn get_dap_binary(
        &mut self,
        _adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        // config.config は JSON 文字列（スキーマ unity.json に従う）
        let unity_config: UnityDebugConfig =
            serde_json::from_str(&config.config).unwrap_or_default();

        // DLL のパスを解決する（worktree経由でシェル環境変数を取得するため渡す）
        let dll_path = find_unity_debug_adapter(user_provided_debug_adapter_path, worktree)?;

        // projectPath が未指定なら worktree のルートを使用
        let project_path = unity_config
            .project_path
            .unwrap_or_else(|| worktree.root_path());

        // Unity DAP サーバーに渡す設定 JSON を組み立てる
        let mut adapter_config = serde_json::json!({
            "request": "attach",
            "projectPath": project_path,
        });

        if let Some(endpoint) = unity_config.end_point {
            adapter_config["endPoint"] = Value::String(endpoint);
        }
        if let Some(log_file) = unity_config.log_file {
            adapter_config["logFile"] = Value::String(log_file);
        }

        let configuration_json = serde_json::to_string(&adapter_config)
            .map_err(|e| format!("Failed to serialize adapter config: {}", e))?;

        Ok(DebugAdapterBinary {
            command: Some("dotnet".to_string()),
            arguments: vec![dll_path],
            envs: vec![],
            cwd: None,
            connection: None,
            request_args: StartDebuggingRequestArguments {
                configuration: configuration_json,
                request: StartDebuggingRequestArgumentsRequest::Attach,
            },
        })
    }

    /// Unity は常に Attach モード（Editor を起動するのではなく、既存プロセスにアタッチ）
    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        _config: Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        Ok(StartDebuggingRequestArgumentsRequest::Attach)
    }

    /// 汎用 DebugConfig → Unity 固有の DebugScenario に変換
    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        match &config.request {
            DebugRequest::Attach(_) => {}
            DebugRequest::Launch(_) => {
                return Err(
                    "Unity debugger only supports 'attach' mode. \
                     Unity Editor must already be running."
                        .to_string(),
                );
            }
        }

        let config_json = serde_json::json!({
            "request": "attach",
            "projectPath": "${ZED_WORKTREE_ROOT}",
        });

        Ok(DebugScenario {
            label: config.label,
            adapter: config.adapter,
            build: None,
            config: serde_json::to_string(&config_json)
                .map_err(|e| format!("Failed to serialize scenario config: {}", e))?,
            tcp_connection: None,
        })
    }
}

/// UnityDebugAdapter.dll のパスを解決する
///
/// 探索順:
///   1. Zed 設定で明示指定されたパス (user_provided_debug_adapter_path)
///   2. visualstudiotoolsforunity.vstuc-* (モダン版 VS Code 拡張)
///   3. unity.unity-debug-* (レガシー拡張)
///
/// 注意: WASM サンドボックス内では std::env::var と Path::exists() が動作しない。
///   - 環境変数は worktree.shell_env() 経由で取得する
///   - ユーザー指定パスの存在確認はスキップ（dotnet が起動失敗すれば明示エラーになる）
fn find_unity_debug_adapter(user_provided: Option<String>, worktree: &Worktree) -> Result<String, String> {
    // 1. ユーザー指定パスが最優先。WASM内では exists() が動かないので存在確認なしで信頼する
    if let Some(path) = user_provided {
        return Ok(path);
    }

    // USERPROFILE / HOME をシェル環境変数から取得
    // （std::env::var は WASM サンドボックスで動作しないため worktree 経由で取得）
    let env = worktree.shell_env();
    let home = env
        .iter()
        .find(|(k, _)| k == "USERPROFILE" || k == "HOME")
        .map(|(_, v)| v.clone())
        .ok_or_else(|| {
            "USERPROFILE / HOME not found in shell environment.\n\
             Please set the DLL path in Zed settings: dap.Unity.binary"
                .to_string()
        })?;

    let extensions_dir = format!("{}/.vscode/extensions", home);

    // 2. モダン版: visualstudiotoolsforunity.vstuc-*
    //    read_dir が WASM サンドボックスで失敗した場合は次のステップへ
    //    存在確認 (exists()) は行わず、パスをそのまま返す
    if let Ok(entries) = std::fs::read_dir(&extensions_dir) {
        let mut candidates: Vec<(String, String)> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("visualstudiotoolsforunity.vstuc-") {
                    let dll =
                        format!("{}/{}/bin/UnityDebugAdapter.dll", extensions_dir, name);
                    Some((name, dll))
                } else {
                    None
                }
            })
            .collect();
        // 降順ソートで最新バージョンを先頭に
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        if let Some((_, dll)) = candidates.into_iter().next() {
            return Ok(dll);
        }
    }

    // 3. レガシー版: unity.unity-debug-*
    if let Ok(entries) = std::fs::read_dir(&extensions_dir) {
        let mut candidates: Vec<(String, String)> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("unity.unity-debug-") {
                    let exe =
                        format!("{}/{}/out/UnityDebug.exe", extensions_dir, name);
                    Some((name, exe))
                } else {
                    None
                }
            })
            .collect();
        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        if let Some((_, exe)) = candidates.into_iter().next() {
            return Ok(exe);
        }
    }

    Err(format!(
        "Unity debug adapter not found.\n\
         Install 'Visual Studio Tools for Unity' VS Code extension, then set:\n\
         Zed settings → dap.Unity.binary = \
         \"{}/visualstudiotoolsforunity.vstuc-<VERSION>/bin/UnityDebugAdapter.dll\"",
        extensions_dir
    ))
}

zed::register_extension!(UnityDebuggerExtension);
