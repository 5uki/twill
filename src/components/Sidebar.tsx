import { Button } from '@fluentui/react-components';
import {
  MailInboxRegular,
  ShieldKeyholeRegular,
  AlertRegular,
  ArchiveRegular,
  PersonAccountsRegular,
  SettingsRegular
} from '@fluentui/react-icons';

interface SidebarProps {
  activeCategory: string;
  onCategoryChange: (category: string) => void;
}

const categories = [
  { id: 'inbox', label: 'All Inbox', icon: <MailInboxRegular /> },
  { id: 'verifications', label: 'Verifications', icon: <ShieldKeyholeRegular /> },
  { id: 'alerts', label: 'Security Alerts', icon: <AlertRegular /> },
  { id: 'archive', label: 'Archive', icon: <ArchiveRegular /> },
];

export function Sidebar({ activeCategory, onCategoryChange }: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="sidebar-group-title">Mailboxes</div>
      
      <div style={{ display: 'flex', flexDirection: 'column', gap: '4px', marginBottom: '16px' }}>
        {categories.map(item => {
          const isActive = activeCategory === item.id;
          return (
            <Button
              key={item.id}
              appearance={isActive ? 'secondary' : 'subtle'}
              icon={item.icon}
              onClick={() => onCategoryChange(item.id)}
              style={{ 
                justifyContent: 'flex-start',
                padding: '8px 12px',
                fontWeight: isActive ? 600 : 400,
                backgroundColor: isActive ? '#e0eaf9' : 'transparent',
                color: isActive ? '#0078d4' : '#374151',
                border: 'none',
                borderRadius: '6px'
              }}
            >
              {item.label}
            </Button>
          );
        })}
      </div>

      <div className="sidebar-group-title">Workspace</div>
      
      <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
        <Button
          appearance={activeCategory === 'accounts' ? 'secondary' : 'subtle'}
          icon={<PersonAccountsRegular />}
          onClick={() => onCategoryChange('accounts')}
          style={{ 
            justifyContent: 'flex-start',
            padding: '8px 12px',
            fontWeight: activeCategory === 'accounts' ? 600 : 400,
            backgroundColor: activeCategory === 'accounts' ? '#e0eaf9' : 'transparent',
            color: activeCategory === 'accounts' ? '#0078d4' : '#374151',
            border: 'none',
            borderRadius: '6px'
          }}
        >
          Accounts
        </Button>
      </div>

      <div style={{ flex: 1 }} />

      <Button
        appearance="subtle"
        icon={<SettingsRegular />}
        style={{ justifyContent: 'flex-start', padding: '8px 12px', color: '#6b7280' }}
      >
        Settings
      </Button>
    </aside>
  );
}
