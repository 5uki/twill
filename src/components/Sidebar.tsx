import { Badge, Button } from "@fluentui/react-components";
import { SettingsRegular } from "@fluentui/react-icons";
import type { JSX } from "react";
import type {
  WorkspaceCategory,
  WorkspaceCategoryGroup,
} from "../lib/workspace-view";

export interface SidebarItem {
  id: WorkspaceCategory;
  label: string;
  badge: number;
  icon: JSX.Element;
}

export interface SidebarGroup {
  id: WorkspaceCategoryGroup;
  label: string;
  items: SidebarItem[];
}

interface SidebarProps {
  activeCategory: WorkspaceCategory;
  groups: SidebarGroup[];
  onCategoryChange: (category: WorkspaceCategory) => void;
}

export function Sidebar({
  activeCategory,
  groups,
  onCategoryChange,
}: SidebarProps) {
  return (
    <aside className="sidebar">
      {groups.map((group) => (
        <section className="sidebar-group" key={group.id}>
          <div className="sidebar-group-title">{group.label}</div>

          <div className="sidebar-nav">
            {group.items.map((item) => {
              const isActive = activeCategory === item.id;

              return (
                <Button
                  key={item.id}
                  appearance={isActive ? "secondary" : "subtle"}
                  icon={item.icon}
                  onClick={() => onCategoryChange(item.id)}
                  style={{
                    justifyContent: "space-between",
                    padding: "8px 12px",
                    fontWeight: isActive ? 600 : 400,
                    backgroundColor: isActive ? "#e0eaf9" : "transparent",
                    color: isActive ? "#0078d4" : "#374151",
                    border: "none",
                    borderRadius: "8px",
                  }}
                >
                  <span>{item.label}</span>
                  <Badge appearance="tint">{item.badge}</Badge>
                </Button>
              );
            })}
          </div>
        </section>
      ))}

      <div style={{ flex: 1 }} />

      <Button
        appearance="subtle"
        icon={<SettingsRegular />}
        style={{ justifyContent: "flex-start", padding: "8px 12px", color: "#6b7280" }}
      >
        设置
      </Button>
    </aside>
  );
}
