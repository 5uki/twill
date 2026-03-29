use crate::domain::error::AppError;
use crate::services::workspace_service;

enum OutputFormat {
    Text,
    Json,
}

pub fn run_from_env() -> Result<String, AppError> {
    run_with_args(std::env::args().skip(1))
}

pub fn run_with_args<I, S>(args: I) -> Result<String, AppError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();

    match args.as_slice() {
        [workspace, bootstrap] if workspace == "workspace" && bootstrap == "bootstrap" => {
            render_workspace_bootstrap(OutputFormat::Text)
        }
        [workspace, bootstrap, flag, _format]
            if workspace == "workspace" && bootstrap == "bootstrap" && flag == "--format" =>
        {
            let format = parse_output_format(args[3].as_str())?;

            render_workspace_bootstrap(format)
        }
        _ => Err(AppError::InvalidCliArgs {
            message: "用法: workspace bootstrap [--format text|json]".to_string(),
        }),
    }
}

fn parse_output_format(value: &str) -> Result<OutputFormat, AppError> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(AppError::UnsupportedFormat {
            format: other.to_string(),
        }),
    }
}

fn render_workspace_bootstrap(format: OutputFormat) -> Result<String, AppError> {
    let snapshot = workspace_service::load_workspace_bootstrap();

    match format {
        OutputFormat::Text => {
            let navigation = snapshot
                .navigation
                .iter()
                .map(|item| format!("- {} ({})", item.label, item.badge))
                .collect::<Vec<_>>()
                .join("\n");

            Ok(format!(
                "Twill workspace bootstrap\n默认视图: Recent verification\n生成时间: {}\n导航:\n{}\n当前选中: {}\n验证码: {}\n链接: {}",
                snapshot.generated_at,
                navigation,
                snapshot.selected_message.subject,
                snapshot
                    .selected_message
                    .extracted_code
                    .as_deref()
                    .unwrap_or("无"),
                snapshot
                    .selected_message
                    .verification_link
                    .as_deref()
                    .unwrap_or("无")
            ))
        }
        OutputFormat::Json => {
            serde_json::to_string_pretty(&snapshot).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run_with_args;
    use crate::domain::error::AppError;

    #[test]
    fn defaults_to_text_output_for_workspace_bootstrap() {
        let output = run_with_args(["workspace", "bootstrap"]).expect("命令应执行成功");

        assert!(
            output.contains("Recent verification"),
            "文本输出至少要包含默认工作台视图"
        );
    }

    #[test]
    fn returns_json_output_when_requested() {
        let output = run_with_args(["workspace", "bootstrap", "--format", "json"])
            .expect("json 输出应执行成功");
        let parsed =
            serde_json::from_str::<serde_json::Value>(&output).expect("输出必须是可解析的 JSON");

        assert_eq!(parsed["default_view"], "recent_verification");
    }

    #[test]
    fn rejects_unsupported_formats() {
        let error = run_with_args(["workspace", "bootstrap", "--format", "yaml"])
            .expect_err("不支持的格式必须报错");

        assert_eq!(
            error,
            AppError::UnsupportedFormat {
                format: "yaml".to_string(),
            }
        );
    }
}
