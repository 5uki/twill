use crate::domain::workspace::MessageCategory;
use crate::domain::workspace::MessageStatus;
use crate::domain::workspace::NavigationItem;
use crate::domain::workspace::WorkspaceBootstrapSnapshot;
use crate::domain::workspace::WorkspaceMessageDetail;
use crate::domain::workspace::WorkspaceMessageGroup;
use crate::domain::workspace::WorkspaceMessageItem;
use crate::domain::workspace::WorkspaceViewId;

pub fn load_snapshot() -> WorkspaceBootstrapSnapshot {
    let pending_item = WorkspaceMessageItem {
        id: "msg_github_security".to_string(),
        subject: "GitHub 安全验证码".to_string(),
        sender: "noreply@github.com".to_string(),
        account_name: "Primary Gmail".to_string(),
        received_at: "2026-03-29T08:58:00Z".to_string(),
        category: MessageCategory::Security,
        status: MessageStatus::Pending,
        has_code: true,
        has_link: false,
        preview: "你的 GitHub 登录验证码是 362149。".to_string(),
    };
    let link_item = WorkspaceMessageItem {
        id: "msg_linear_verify".to_string(),
        subject: "Linear 验证链接".to_string(),
        sender: "hello@linear.app".to_string(),
        account_name: "Work Outlook".to_string(),
        received_at: "2026-03-29T08:41:00Z".to_string(),
        category: MessageCategory::Registration,
        status: MessageStatus::Pending,
        has_code: false,
        has_link: true,
        preview: "点击邮件中的安全链接完成登录。".to_string(),
    };
    let processed_item = WorkspaceMessageItem {
        id: "msg_notion_welcome".to_string(),
        subject: "Notion 注册确认".to_string(),
        sender: "team@makenotion.com".to_string(),
        account_name: "Primary Gmail".to_string(),
        received_at: "2026-03-29T07:12:00Z".to_string(),
        category: MessageCategory::Marketing,
        status: MessageStatus::Processed,
        has_code: false,
        has_link: true,
        preview: "已完成注册，可继续打开欢迎链接。".to_string(),
    };

    WorkspaceBootstrapSnapshot {
        app_name: "Twill".to_string(),
        generated_at: "2026-03-29T09:00:00Z".to_string(),
        default_view: WorkspaceViewId::RecentVerification,
        navigation: vec![
            NavigationItem {
                id: WorkspaceViewId::RecentVerification,
                label: "Recent verification".to_string(),
                badge: 12,
            },
            NavigationItem {
                id: WorkspaceViewId::AllInbox,
                label: "All inbox".to_string(),
                badge: 128,
            },
            NavigationItem {
                id: WorkspaceViewId::SiteList,
                label: "Sites".to_string(),
                badge: 18,
            },
            NavigationItem {
                id: WorkspaceViewId::Accounts,
                label: "Accounts".to_string(),
                badge: 3,
            },
        ],
        message_groups: vec![
            WorkspaceMessageGroup {
                id: "pending".to_string(),
                label: "待处理".to_string(),
                items: vec![pending_item.clone(), link_item],
            },
            WorkspaceMessageGroup {
                id: "processed".to_string(),
                label: "已处理".to_string(),
                items: vec![processed_item],
            },
        ],
        selected_message: WorkspaceMessageDetail {
            id: pending_item.id,
            subject: pending_item.subject,
            sender: pending_item.sender,
            account_name: pending_item.account_name,
            received_at: pending_item.received_at,
            category: pending_item.category,
            status: pending_item.status,
            site_hint: "github.com".to_string(),
            summary: "这是本轮 M0 的静态样例数据，用于驱动 Recent verification 工作台壳层。"
                .to_string(),
            extracted_code: Some("362149".to_string()),
            verification_link: Some("https://github.com/login/device".to_string()),
        },
    }
}
