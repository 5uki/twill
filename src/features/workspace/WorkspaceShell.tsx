import {
  Badge,
  Button,
  Caption1Strong,
  Input,
  Subtitle2Stronger,
  Text,
  Title3,
} from "@fluentui/react-components";
import { useState, type CSSProperties } from "react";
import { AccountsDetailPanel } from "../accounts/AccountsDetailPanel";
import { AccountsWorkspacePanel } from "../accounts/AccountsWorkspacePanel";
import { useAccountsOnboarding } from "../accounts/useAccountsOnboarding";
import type { WorkspaceViewModel } from "./view-model";

interface WorkspaceShellProps {
  viewModel: WorkspaceViewModel;
}

export function WorkspaceShell({ viewModel }: WorkspaceShellProps) {
  const initialView =
    viewModel.navigation.find((item) => item.isActive)?.id ?? "recent_verification";
  const [activeView, setActiveView] = useState(initialView);
  const accounts = useAccountsOnboarding();

  const navigation = viewModel.navigation.map((item) => {
    if (item.id === "accounts") {
      return {
        ...item,
        badge: accounts.accountCount,
        isActive: item.id === activeView,
      };
    }

    return {
      ...item,
      isActive: item.id === activeView,
    };
  });

  const activeNavigation = navigation.find((item) => item.id === activeView) ?? navigation[0];
  const detailMotionKey =
    activeView === "accounts"
      ? `accounts-${accounts.lastSavedAccount?.id ?? "none"}-${accounts.lastTestResult?.status ?? "idle"}-${accounts.saveState}-${accounts.testState}`
      : activeView;

  return (
    <main className="workspace-shell">
      <div className="workspace-shell__frame">
        <header className="workspace-shell__topbar" style={panelMotionStyle(0)}>
          <section className="workspace-shell__identity">
            <span className="workspace-shell__eyebrow">Twill / M0 + M1</span>
            <div className="workspace-shell__title-row">
              <Title3 className="workspace-shell__title">{viewModel.appName}</Title3>
              <Badge appearance="tint" color="brand">
                {activeNavigation.label}
              </Badge>
            </div>
            <Text className="workspace-shell__subtitle">{viewModel.subtitle}</Text>
          </section>

          <section className="workspace-shell__inputs">
            <Input appearance="filled-darker" placeholder="当前站点，例如 github.com" />
            <Input appearance="filled-darker" placeholder="全局搜索邮件、站点或账号" />
          </section>

          <section className="workspace-shell__account">
            <div className="workspace-shell__meta">
              <Badge appearance="filled" color="brand">
                CLI / Tauri 共用服务
              </Badge>
              <Badge appearance="outline">快照时间 {viewModel.generatedAtLabel}</Badge>
            </div>
            <Button appearance="secondary">同步入口</Button>
          </section>
        </header>

        <div className="workspace-shell__body">
          <aside
            className="workspace-shell__panel workspace-shell__panel--nav"
            style={panelMotionStyle(1)}
          >
            <div className="workspace-shell__section-head">
              <div>
                <div className="workspace-shell__section-title">工作台导航</div>
                <Text className="workspace-shell__section-desc">产品入口优先，不暴露传统邮箱树。</Text>
              </div>
            </div>
            <div className="workspace-shell__nav-list">
              {navigation.map((item, index) => (
                <button
                  key={item.id}
                  type="button"
                  className={[
                    "workspace-shell__nav-item",
                    item.isActive ? "workspace-shell__nav-item--active" : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                  onClick={() => setActiveView(item.id)}
                  style={itemMotionStyle(index)}
                >
                  <div className="workspace-shell__nav-topline">
                    <span className="workspace-shell__nav-label">{item.label}</span>
                    <Badge appearance={item.isActive ? "filled" : "outline"} color="brand">
                      {item.badge}
                    </Badge>
                  </div>
                  <Text>{navDescription(item.id, item.isActive)}</Text>
                </button>
              ))}
            </div>
          </aside>

          <section
            className="workspace-shell__panel workspace-shell__panel--list"
            style={panelMotionStyle(2)}
          >
            {activeView === "accounts" ? (
              <AccountsWorkspacePanel
                form={accounts.form}
                accountsState={accounts.accountsState}
                saveState={accounts.saveState}
                testState={accounts.testState}
                saveError={accounts.saveError}
                testError={accounts.testError}
                lastSavedAccount={accounts.lastSavedAccount}
                lastTestResult={accounts.lastTestResult}
                onFieldChange={accounts.updateField}
                onSave={() => void accounts.saveAccountDraft()}
                onTest={() => void accounts.runConnectionTest()}
                onRefresh={() => void accounts.loadAccounts()}
              />
            ) : (
              <div key={activeView} className="workspace-shell__content-switch">
                <div className="workspace-shell__section-head">
                  <div>
                    <div className="workspace-shell__section-title">工作台列表</div>
                    <Text className="workspace-shell__section-desc">
                      最新优先，高密度显示账号归属、动作信号和摘要。
                    </Text>
                  </div>
                  <Badge appearance="tint">默认按时间倒序</Badge>
                </div>

                <div className="workspace-shell__groups">
                  {viewModel.groups.map((group, groupIndex) => (
                    <section key={group.id} className="workspace-shell__group">
                      <div className="workspace-shell__section-head">
                        <Subtitle2Stronger>{group.label}</Subtitle2Stronger>
                        <Caption1Strong>{group.count} 封</Caption1Strong>
                      </div>
                      <div className="workspace-shell__group-items">
                        {group.items.map((item, itemIndex) => (
                          <article
                            key={item.id}
                            className={[
                              "workspace-shell__message-card",
                              item.isSelected ? "workspace-shell__message-card--selected" : "",
                              item.status === "processed"
                                ? "workspace-shell__message-card--processed"
                                : "",
                            ]
                              .filter(Boolean)
                              .join(" ")}
                            style={itemMotionStyle(groupIndex + itemIndex)}
                          >
                            <div className="workspace-shell__message-topline">
                              <span className="workspace-shell__message-subject">{item.subject}</span>
                              <Badge appearance="tint">{item.categoryLabel}</Badge>
                              <Badge appearance="outline">{item.receivedAt}</Badge>
                            </div>
                            <Text>{item.accountName}</Text>
                            <Text className="workspace-shell__message-preview">{item.preview}</Text>
                            <div className="workspace-shell__message-topline">
                              <Badge appearance="filled" color="informative">
                                {item.sender}
                              </Badge>
                              {item.flags.map((flag) => (
                                <Badge key={flag} appearance="outline" color="warning">
                                  {flag}
                                </Badge>
                              ))}
                            </div>
                          </article>
                        ))}
                      </div>
                    </section>
                  ))}
                </div>
              </div>
            )}
          </section>

          <section
            className="workspace-shell__panel workspace-shell__panel--detail"
            style={panelMotionStyle(3)}
          >
            {activeView === "accounts" ? (
              <div key={detailMotionKey}>
                <AccountsDetailPanel
                  accountCount={accounts.accountCount}
                  lastSavedAccount={accounts.lastSavedAccount}
                  lastTestResult={accounts.lastTestResult}
                  saveState={accounts.saveState}
                  testState={accounts.testState}
                />
              </div>
            ) : (
              <div key={detailMotionKey} className="workspace-shell__content-switch">
                <div className="workspace-shell__section-head">
                  <div>
                    <div className="workspace-shell__section-title">右侧详情</div>
                    <Text className="workspace-shell__section-desc">
                      提取结果优先，原始邮件入口作为稳定兜底。
                    </Text>
                  </div>
                  <Badge appearance="filled" color="success">
                    {viewModel.detail.statusLabel}
                  </Badge>
                </div>

                <div className="workspace-shell__action-grid">
                  {viewModel.detail.actions.map((action, index) => (
                    <article
                      key={action.id}
                      className="workspace-shell__action-card"
                      style={itemMotionStyle(index)}
                    >
                      <div className="workspace-shell__action-meta">
                        <span className="workspace-shell__action-title">{action.label}</span>
                        {action.disabled ? (
                          <Badge appearance="outline">不可用</Badge>
                        ) : (
                          <Badge appearance="tint" color="brand">
                            就绪
                          </Badge>
                        )}
                      </div>
                      <Text className="workspace-shell__action-body">{action.body}</Text>
                      <Button appearance={action.appearance} disabled={action.disabled}>
                        {action.label}
                      </Button>
                    </article>
                  ))}
                </div>

                <div className="workspace-shell__detail-grid">
                  <article className="workspace-shell__detail-card workspace-shell__detail-card--signal">
                    <div className="workspace-shell__detail-metadata">
                      <Badge appearance="filled" color="brand">
                        {viewModel.detail.categoryLabel}
                      </Badge>
                      <Badge appearance="outline">{viewModel.detail.siteHint}</Badge>
                      <Badge appearance="outline">{viewModel.detail.receivedAt}</Badge>
                    </div>
                    <div className="workspace-shell__detail-value">{viewModel.detail.subject}</div>
                    <Text>{viewModel.detail.accountName}</Text>
                    <Text>{viewModel.detail.sender}</Text>
                  </article>

                  <article className="workspace-shell__detail-card">
                    <div className="workspace-shell__detail-key">提取到的验证码</div>
                    <div className="workspace-shell__detail-code">
                      {viewModel.detail.extractedCode ?? "无"}
                    </div>
                  </article>

                  <article className="workspace-shell__detail-card">
                    <div className="workspace-shell__detail-key">验证链接</div>
                    <div className="workspace-shell__detail-link">
                      {viewModel.detail.verificationLink ?? "当前邮件没有验证链接"}
                    </div>
                  </article>

                  <article className="workspace-shell__detail-card">
                    <div className="workspace-shell__detail-key">摘要</div>
                    <Text className="workspace-shell__detail-summary">
                      {viewModel.detail.summary}
                    </Text>
                  </article>
                </div>

                <Text className="workspace-shell__footer-note">
                  当前页面由 Rust 静态快照驱动，M1 已开始接入账户 onboarding，后续 M2 再把同步与真实连接接上来。
                </Text>
              </div>
            )}
          </section>
        </div>
      </div>
    </main>
  );
}

function navDescription(id: string, isActive: boolean) {
  if (id === "accounts") {
    return isActive ? "M1 接入首切片已激活" : "打开账户接入面板";
  }

  return isActive ? "当前主视图" : "预留后续切换入口";
}

function panelMotionStyle(index: number): CSSProperties {
  return {
    ["--panel-index" as string]: index,
  };
}

function itemMotionStyle(index: number): CSSProperties {
  return {
    ["--stagger-index" as string]: index,
  };
}
