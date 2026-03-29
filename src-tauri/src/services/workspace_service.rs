use crate::domain::workspace::WorkspaceBootstrapSnapshot;
use crate::infra::static_workspace;

pub fn load_workspace_bootstrap() -> WorkspaceBootstrapSnapshot {
    static_workspace::load_snapshot()
}

#[cfg(test)]
mod tests {
    use super::load_workspace_bootstrap;
    use crate::domain::workspace::WorkspaceViewId;

    #[test]
    fn loads_recent_verification_as_default_view() {
        let snapshot = load_workspace_bootstrap();

        assert_eq!(snapshot.default_view, WorkspaceViewId::RecentVerification);
    }

    #[test]
    fn loads_navigation_and_message_groups() {
        let snapshot = load_workspace_bootstrap();

        assert!(
            snapshot
                .navigation
                .iter()
                .any(|item| item.id == WorkspaceViewId::RecentVerification),
            "导航里至少要包含 Recent verification"
        );
        assert!(
            !snapshot.message_groups.is_empty(),
            "M0 的启动快照至少要提供一组列表数据"
        );
    }
}
