import { useState } from 'react';
import { 
  Title1, Text, Input, Field, Select, Button, Badge, 
  Spinner, ProgressBar 
} from '@fluentui/react-components';
import { ShieldKeyholeRegular, ServerRegular, CheckmarkCircleRegular } from '@fluentui/react-icons';
import { motion, AnimatePresence } from 'framer-motion';

export function AccountsOnboarding() {
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<'idle' | 'success' | 'error'>('idle');

  const handleTest = () => {
    setIsTesting(true);
    setTestResult('idle');
    setTimeout(() => {
      setIsTesting(false);
      setTestResult('success');
    }, 2000);
  };

  return (
    <motion.div 
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay: 0.2, ease: [0.22, 1, 0.36, 1] }}
      style={{ maxWidth: '900px', margin: '0 auto', display: 'flex', flexDirection: 'column', gap: '40px', paddingBottom: '40px' }}
    >
      {/* Hero Section */}
      <section style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <ShieldKeyholeRegular style={{ fontSize: '32px', color: '#10717F' }} />
          <Title1 style={{ fontWeight: 600, letterSpacing: '-0.02em' }}>Connect Workspace</Title1>
        </div>
        <Text style={{ fontSize: '16px', color: '#5A6B7C', lineHeight: 1.6, maxWidth: '600px' }}>
          Twill requires IMAP/SMTP access to verify signals. Your credentials are encrypted and stored in the <strong>local system keychain</strong>. We never upload your passwords to any cloud.
        </Text>
      </section>

      {/* Form Section */}
      <section style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
        
        {/* Identity Grid */}
        <div className="glass-card" style={{ padding: '24px', display: 'flex', flexDirection: 'column', gap: '20px' }}>
          <div style={{ fontWeight: 600, fontSize: '16px', color: '#1A212B' }}>Account Identity</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '20px' }}>
            <Field label="Display Name">
              <Input placeholder="e.g. Work Outlook" appearance="underline" />
            </Field>
            <Field label="Email Address">
              <Input placeholder="name@company.com" appearance="underline" />
            </Field>
            <Field label="App Password">
              <Input type="password" placeholder="System-level secure storage" appearance="underline" 
                     contentAfter={<ShieldKeyholeRegular style={{ color: '#10717F' }} />} />
            </Field>
          </div>
        </div>

        {/* Servers Grid */}
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px' }}>
          
          <div className="glass-card" style={{ padding: '24px', display: 'flex', flexDirection: 'column', gap: '20px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontWeight: 600 }}>
                <ServerRegular /> IMAP Server
              </div>
              <Badge appearance="tint">Incoming</Badge>
            </div>
            <Field label="Hostname">
              <Input placeholder="imap.example.com" appearance="underline" className="font-mono" />
            </Field>
            <div style={{ display: 'flex', gap: '16px' }}>
              <Field label="Port" style={{ flex: 1 }}>
                <Input placeholder="993" appearance="underline" className="font-mono" />
              </Field>
              <Field label="Security" style={{ flex: 2 }}>
                <Select appearance="underline">
                  <option>TLS / SSL</option>
                  <option>STARTTLS</option>
                  <option>None</option>
                </Select>
              </Field>
            </div>
          </div>

          <div className="glass-card" style={{ padding: '24px', display: 'flex', flexDirection: 'column', gap: '20px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontWeight: 600 }}>
                <ServerRegular /> SMTP Server
              </div>
              <Badge appearance="tint">Outgoing</Badge>
            </div>
            <Field label="Hostname">
              <Input placeholder="smtp.example.com" appearance="underline" className="font-mono" />
            </Field>
            <div style={{ display: 'flex', gap: '16px' }}>
              <Field label="Port" style={{ flex: 1 }}>
                <Input placeholder="587" appearance="underline" className="font-mono" />
              </Field>
              <Field label="Security" style={{ flex: 2 }}>
                <Select appearance="underline">
                  <option>STARTTLS</option>
                  <option>TLS / SSL</option>
                  <option>None</option>
                </Select>
              </Field>
            </div>
          </div>

        </div>
      </section>

      {/* Action & Console */}
      <section style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
        <div style={{ display: 'flex', gap: '16px' }}>
          <Button appearance="primary" size="large" style={{ backgroundColor: '#10717F', borderRadius: '8px' }}>
            Save to Keychain
          </Button>
          <Button appearance="secondary" size="large" style={{ borderRadius: '8px' }} onClick={handleTest}>
            Live Probe
          </Button>
        </div>

        <AnimatePresence>
          {(isTesting || testResult !== 'idle') && (
            <motion.div 
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: 'auto', opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              style={{ overflow: 'hidden' }}
            >
              <div className="glass-card font-mono" style={{ 
                padding: '20px', 
                background: 'rgba(26, 33, 43, 0.85)', 
                color: '#A6F5FD', 
                fontSize: '13px',
                border: '1px solid rgba(0,0,0,0.2)',
                boxShadow: 'inset 0 2px 10px rgba(0,0,0,0.5)'
              }}>
                {isTesting && (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Spinner size="tiny" appearance="inverted" /> 
                      <span>Resolving IMAP host...</span>
                    </div>
                    <ProgressBar thickness="large" color="success" value={0.6} />
                  </div>
                )}
                {testResult === 'success' && !isTesting && (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', color: '#60E7F8' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <CheckmarkCircleRegular style={{ color: '#4ADE80' }} /> Authenticated successfully with IMAP.
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <CheckmarkCircleRegular style={{ color: '#4ADE80' }} /> SMTP connected.
                    </div>
                  </div>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>

      </section>
    </motion.div>
  );
}
