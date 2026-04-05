import { Input, Avatar, Badge } from '@fluentui/react-components';
import { SearchRegular, CloudSyncRegular } from '@fluentui/react-icons';
import { motion } from 'framer-motion';

export function CommandHeader({ viewTitle }: { viewTitle: string }) {
  return (
    <motion.header 
      className="command-header"
      initial={{ y: -20, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ duration: 0.4, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
    >
      <div className="command-header-left">
        <span style={{ color: '#8895A3' }}>/</span>
        <span style={{ color: '#10717F' }}>{viewTitle}</span>
      </div>

      <div className="command-input">
        <Input 
          contentBefore={<SearchRegular />}
          placeholder="Type / to search or filter..."
          appearance="filled-darker"
          style={{ width: '100%', borderRadius: '10px', border: '1px solid rgba(0,0,0,0.06)', background: 'rgba(255,255,255,0.8)' }}
        />
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
        <Badge appearance="tint" color="success" icon={<CloudSyncRegular />}>
          Synced
        </Badge>
        <Avatar name="Artaphy" size={32} />
      </div>
    </motion.header>
  );
}
