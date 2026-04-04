use crate::domain::account::{
    AccountConnectionStatus, AccountConnectionTestInput, AccountConnectionTestResult,
    AddAccountInput, MailSecurity, MailServerConfig,
};
use crate::domain::error::AppError;
use crate::infra::account_preflight::RuleBasedAccountConnectionTester;
use crate::infra::account_store::JsonFileAccountRepository;
use crate::services::{account_service, workspace_service};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    run_with_args_and_store_path(args, default_store_path())
}

fn run_with_args_and_store_path<I, S>(args: I, store_path: PathBuf) -> Result<String, AppError>
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
        [workspace, bootstrap, flag, value]
            if workspace == "workspace" && bootstrap == "bootstrap" && flag == "--format" =>
        {
            render_workspace_bootstrap(parse_output_format(value)?)
        }
        [account, list] if account == "account" && list == "list" => {
            render_account_list(OutputFormat::Text, &store_path)
        }
        [account, list, rest @ ..] if account == "account" && list == "list" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_account_list(format, &store_path)
        }
        [account, add, rest @ ..] if account == "account" && add == "add" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let input = parse_add_account_input(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_add_account(format, &store_path, input)
        }
        [account, test, rest @ ..] if account == "account" && test == "test" => {
            let mut flags = parse_flags(rest)?;
            let format = take_output_format(&mut flags)?;
            let input = parse_account_test_input(&mut flags)?;

            ensure_no_unknown_flags(&flags)?;
            render_account_test(format, input)
        }
        _ => Err(AppError::InvalidCliArgs {
            message: concat!(
                "用法:\n",
                "  workspace bootstrap [--format text|json]\n",
                "  account list [--format text|json]\n",
                "  account add --name <name> --email <email> --login <login> ",
                "--imap-host <host> --imap-port <port> --imap-security <none|start_tls|tls> ",
                "--smtp-host <host> --smtp-port <port> --smtp-security <none|start_tls|tls> ",
                "[--format text|json]\n",
                "  account test --name <name> --email <email> --login <login> ",
                "--imap-host <host> --imap-port <port> --imap-security <none|start_tls|tls> ",
                "--smtp-host <host> --smtp-port <port> --smtp-security <none|start_tls|tls> ",
                "[--format text|json]"
            )
            .to_string(),
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

fn render_account_list(format: OutputFormat, store_path: &Path) -> Result<String, AppError> {
    let repository = JsonFileAccountRepository::new(store_path);
    let accounts = account_service::list_accounts(&repository)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&accounts).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => {
            if accounts.is_empty() {
                Ok("当前没有已保存的账户配置。".to_string())
            } else {
                let lines = accounts
                    .iter()
                    .map(|account| {
                        format!(
                            "- {} <{}> | IMAP {}:{} {:?} | SMTP {}:{} {:?}",
                            account.display_name,
                            account.email,
                            account.imap.host,
                            account.imap.port,
                            account.imap.security,
                            account.smtp.host,
                            account.smtp.port,
                            account.smtp.security
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(format!("已保存账户:\n{lines}"))
            }
        }
    }
}

fn render_add_account(
    format: OutputFormat,
    store_path: &Path,
    input: AddAccountInput,
) -> Result<String, AppError> {
    let repository = JsonFileAccountRepository::new(store_path);
    let account = account_service::add_account(&repository, input)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&account).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format!(
            "账户已保存\nID: {}\n名称: {}\n邮箱: {}\nIMAP: {}:{} ({:?})\nSMTP: {}:{} ({:?})",
            account.id,
            account.display_name,
            account.email,
            account.imap.host,
            account.imap.port,
            account.imap.security,
            account.smtp.host,
            account.smtp.port,
            account.smtp.security
        )),
    }
}

fn render_account_test(
    format: OutputFormat,
    input: AccountConnectionTestInput,
) -> Result<String, AppError> {
    let tester = RuleBasedAccountConnectionTester;
    let result = account_service::test_account_connection(&tester, input)?;

    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&result).map_err(|error| AppError::Serialization {
                message: error.to_string(),
            })
        }
        OutputFormat::Text => Ok(format_account_test_result(&result)),
    }
}

fn format_account_test_result(result: &AccountConnectionTestResult) -> String {
    let status = match result.status {
        AccountConnectionStatus::Passed => "通过",
        AccountConnectionStatus::Failed => "失败",
    };

    let checks = result
        .checks
        .iter()
        .map(|check| {
            let target = match check.target {
                crate::domain::account::AccountConnectionCheckTarget::Identity => "Identity",
                crate::domain::account::AccountConnectionCheckTarget::Imap => "IMAP",
                crate::domain::account::AccountConnectionCheckTarget::Smtp => "SMTP",
            };
            let check_status = match check.status {
                AccountConnectionStatus::Passed => "通过",
                AccountConnectionStatus::Failed => "失败",
            };

            format!("- {target}: {check_status} | {}", check.message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "账户连接预检\n状态: {status}\n结论: {}\n{checks}",
        result.summary
    )
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, AppError> {
    let mut flags = BTreeMap::new();
    let mut index = 0;

    while index < args.len() {
        let flag = &args[index];

        if !flag.starts_with("--") {
            return Err(AppError::InvalidCliArgs {
                message: format!("无法识别的位置参数: {flag}"),
            });
        }

        let value = args
            .get(index + 1)
            .ok_or_else(|| AppError::InvalidCliArgs {
                message: format!("参数 {flag} 缺少取值"),
            })?;

        if value.starts_with("--") {
            return Err(AppError::InvalidCliArgs {
                message: format!("参数 {flag} 缺少取值"),
            });
        }

        flags.insert(flag.clone(), value.clone());
        index += 2;
    }

    Ok(flags)
}

fn take_output_format(flags: &mut BTreeMap<String, String>) -> Result<OutputFormat, AppError> {
    match flags.remove("--format") {
        Some(value) => parse_output_format(&value),
        None => Ok(OutputFormat::Text),
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

fn parse_add_account_input(
    flags: &mut BTreeMap<String, String>,
) -> Result<AddAccountInput, AppError> {
    Ok(AddAccountInput {
        display_name: take_required_flag(flags, "--name")?,
        email: take_required_flag(flags, "--email")?,
        login: take_required_flag(flags, "--login")?,
        imap: MailServerConfig {
            host: take_required_flag(flags, "--imap-host")?,
            port: parse_port(&take_required_flag(flags, "--imap-port")?, "imap.port")?,
            security: parse_security(&take_required_flag(flags, "--imap-security")?)?,
        },
        smtp: MailServerConfig {
            host: take_required_flag(flags, "--smtp-host")?,
            port: parse_port(&take_required_flag(flags, "--smtp-port")?, "smtp.port")?,
            security: parse_security(&take_required_flag(flags, "--smtp-security")?)?,
        },
    })
}

fn parse_account_test_input(
    flags: &mut BTreeMap<String, String>,
) -> Result<AccountConnectionTestInput, AppError> {
    Ok(AccountConnectionTestInput {
        display_name: take_required_flag(flags, "--name")?,
        email: take_required_flag(flags, "--email")?,
        login: take_required_flag(flags, "--login")?,
        imap: MailServerConfig {
            host: take_required_flag(flags, "--imap-host")?,
            port: parse_port(&take_required_flag(flags, "--imap-port")?, "imap.port")?,
            security: parse_security(&take_required_flag(flags, "--imap-security")?)?,
        },
        smtp: MailServerConfig {
            host: take_required_flag(flags, "--smtp-host")?,
            port: parse_port(&take_required_flag(flags, "--smtp-port")?, "smtp.port")?,
            security: parse_security(&take_required_flag(flags, "--smtp-security")?)?,
        },
    })
}

fn take_required_flag(flags: &mut BTreeMap<String, String>, key: &str) -> Result<String, AppError> {
    flags.remove(key).ok_or_else(|| AppError::InvalidCliArgs {
        message: format!("缺少参数: {key}"),
    })
}

fn parse_port(value: &str, field: &str) -> Result<u16, AppError> {
    value.parse::<u16>().map_err(|_| AppError::Validation {
        field: field.to_string(),
        message: format!("端口必须是 1 到 65535 之间的整数，收到 {value}"),
    })
}

fn parse_security(value: &str) -> Result<MailSecurity, AppError> {
    match value {
        "none" => Ok(MailSecurity::None),
        "start_tls" => Ok(MailSecurity::StartTls),
        "tls" => Ok(MailSecurity::Tls),
        other => Err(AppError::Validation {
            field: "security".to_string(),
            message: format!("不支持的安全策略: {other}"),
        }),
    }
}

fn ensure_no_unknown_flags(flags: &BTreeMap<String, String>) -> Result<(), AppError> {
    if flags.is_empty() {
        Ok(())
    } else {
        let unknown = flags.keys().cloned().collect::<Vec<_>>().join(", ");

        Err(AppError::InvalidCliArgs {
            message: format!("存在未识别参数: {unknown}"),
        })
    }
}

fn default_store_path() -> PathBuf {
    crate::infra::account_store::default_account_store_path()
}

#[cfg(test)]
mod tests {
    use super::run_with_args_and_store_path;
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn defaults_to_text_output_for_workspace_bootstrap() {
        let output = run_with_args_and_store_path(["workspace", "bootstrap"], unique_store_path())
            .expect("命令应执行成功");

        assert!(
            output.contains("Recent verification"),
            "文本输出至少要包含默认工作台视图"
        );
    }

    #[test]
    fn persists_account_between_add_and_list() {
        let store_path = unique_store_path();

        run_with_args_and_store_path(
            [
                "account",
                "add",
                "--name",
                "Primary Gmail",
                "--email",
                "primary@example.com",
                "--login",
                "primary@example.com",
                "--imap-host",
                "imap.example.com",
                "--imap-port",
                "993",
                "--imap-security",
                "tls",
                "--smtp-host",
                "smtp.example.com",
                "--smtp-port",
                "587",
                "--smtp-security",
                "start_tls",
                "--format",
                "json",
            ],
            store_path.clone(),
        )
        .expect("新增账户应成功");

        let output =
            run_with_args_and_store_path(["account", "list", "--format", "json"], store_path)
                .expect("列出账户应成功");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");

        assert_eq!(parsed.as_array().map(|items| items.len()), Some(1));
    }

    #[test]
    fn reports_failed_preflight_for_mismatched_ports() {
        let output = run_with_args_and_store_path(
            [
                "account",
                "test",
                "--name",
                "Primary Gmail",
                "--email",
                "primary@example.com",
                "--login",
                "primary@example.com",
                "--imap-host",
                "imap.example.com",
                "--imap-port",
                "143",
                "--imap-security",
                "tls",
                "--smtp-host",
                "smtp.example.com",
                "--smtp-port",
                "587",
                "--smtp-security",
                "start_tls",
                "--format",
                "json",
            ],
            unique_store_path(),
        )
        .expect("连接预检命令应返回结构化结果");
        let parsed = serde_json::from_str::<Value>(&output).expect("输出必须是合法 JSON");

        assert_eq!(parsed["status"], "failed");
    }

    #[test]
    fn rejects_unsupported_formats() {
        let error = run_with_args_and_store_path(
            ["workspace", "bootstrap", "--format", "yaml"],
            unique_store_path(),
        )
        .expect_err("不支持的格式必须报错");

        assert_eq!(
            error,
            crate::domain::error::AppError::UnsupportedFormat {
                format: "yaml".to_string(),
            }
        );
    }

    fn unique_store_path() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间应晚于 epoch")
            .as_nanos();

        std::env::temp_dir()
            .join("twill-tests")
            .join(format!("cli-accounts-{suffix}.json"))
    }
}
