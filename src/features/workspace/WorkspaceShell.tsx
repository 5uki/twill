import {
  Badge,
  Button,
  Caption1Strong,
  Input,
  Subtitle2Stronger,
  Text,
  Title3,
} from "@fluentui/react-components";
import type { WorkspaceViewModel } from "./view-model";

interface WorkspaceShellProps {
  viewModel: WorkspaceViewModel;
}

export function WorkspaceShell({ viewModel }: WorkspaceShellProps) {
  return (
    <main className="workspace-shell">
      <div className="workspace-shell__frame">
        <header className="workspace-shell__topbar">
          <section className="workspace-shell__identity">
            <span className="workspace-shell__eyebrow">Twill / M0</span>
            <div className="workspace-shell__title-row">
              <Title3 className="workspace-shell__title">{viewModel.appName}</Title3>
              <Badge appearance="tint" color="brand">
                {viewModel.title}
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
                CLI / Tauri 共用快照
              </Badge>
              <Badge appearance="outline">快照时间 {viewModel.generatedAtLabel}</Badge>
            </div>
            <Button appearance="secondary">同步入口</Button>
          </section>
        </header>

        <div className="workspace-shell__body">
          <aside className="workspace-shell__panel workspace-shell__panel--nav">
            <div className="workspace-shell__section-head">
              <div>
                <div className="workspace-shell__section-title">工作台导航</div>
                <Text className="workspace-shell__section-desc">产品入口优先，不暴露传统邮箱树。</Text>
              </div>
            </div>
            <div className="workspace-shell__nav-list">
              {viewModel.navigation.map((item) => (
                <article
                  key={item.id}
                  className={[
                    "workspace-shell__nav-item",
                    item.isActive ? "workspace-shell__nav-item--active" : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                >
                  <div className="workspace-shell__nav-topline">
                    <span className="workspace-shell__nav-label">{item.label}</span>
                    <Badge appearance={item.isActive ? "filled" : "outline"} color="brand">
                      {item.badge}
                    </Badge>
                  </div>
                  <Text>{item.isActive ? "默认首页" : "预留后续切换入口"}</Text>
                </article>
              ))}
            </div>
          </aside>

          <section className="workspace-shell__panel workspace-shell__panel--list">
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
              {viewModel.groups.map((group) => (
                <section key={group.id} className="workspace-shell__group">
                  <div className="workspace-shell__section-head">
                    <Subtitle2Stronger>{group.label}</Subtitle2Stronger>
                    <Caption1Strong>{group.count} 封</Caption1Strong>
                  </div>
                  <div className="workspace-shell__group-items">
                    {group.items.map((item) => (
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
          </section>

          <section className="workspace-shell__panel workspace-shell__panel--detail">
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
              {viewModel.detail.actions.map((action) => (
                <article key={action.id} className="workspace-shell__action-card">
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
                <Text className="workspace-shell__detail-summary">{viewModel.detail.summary}</Text>
              </article>
            </div>

            <Text className="workspace-shell__footer-note">
              当前页面由 Rust 静态快照驱动，后续 M1/M2 将把账户接入与同步替换进来。
            </Text>
          </section>
        </div>
      </div>
    </main>
  );
}
