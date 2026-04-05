import { Text, Avatar } from '@fluentui/react-components';
import {
  OpenRegular,
  DismissRegular,
  ChevronLeftRegular,
  ChevronRightRegular,
  Open16Filled,
} from '@fluentui/react-icons';
import {
  useState,
  useRef,
  useEffect,
  useId,
  type FocusEvent,
  type MouseEvent,
} from 'react';
import { ExtractTooltipPortal } from './ExtractTooltipPortal';
import { getCircularAvatarMetrics } from './extract-geometry';
import { getExtractActionHint } from './extract-tooltip';

const mockMails = [
  { id: '1', sender: 'GitHub', subject: '[GitHub] Please verify your device', preview: 'Verification code: 482910. This code expires in 10 minutes.', time: 'Just now', type: 'code', unread: true },
  { id: '2', sender: 'Stripe', subject: 'Confirm your login to Stripe', preview: 'Click the link below to securely log in to your Stripe dashboard.', time: '10 mins ago', type: 'link', unread: true },
  { id: '3', sender: 'Steam', subject: 'Steam Guard Computer Authorization', preview: 'Here is the Steam Guard code you need to login to account artaphy...', time: '1 hour ago', type: 'code', unread: false },
  { id: '4', sender: 'AWS', subject: 'AWS Notification - Root Account Login', preview: 'Your AWS account root user has logged in from a new IP.', time: '2 hours ago', type: 'alert', unread: false },
];

type ExtractItem = {
  id: string;
  sender: string;
  type: 'code' | 'link';
  value: string;
  label: string;
  progress: number;
  expiresLabel: string;
};

const mockExtractedCodes: ExtractItem[] = [
  { id: 'c1', sender: 'GitHub', type: 'code', value: '482910', label: '', progress: 0.9, expiresLabel: '9m' },
  { id: 'c2', sender: 'Stripe', type: 'link', value: '#', label: 'Sign In', progress: 0.4, expiresLabel: '4m' },
  { id: 'c3', sender: 'Steam', type: 'code', value: 'R7KV2', label: '', progress: 0.1, expiresLabel: '1m' },
  { id: 'c4', sender: 'Epic', type: 'code', value: 'A7X9-B2C1', label: '', progress: 0.75, expiresLabel: '12m' },
  { id: 'c5', sender: 'Vercel', type: 'code', value: '912384', label: '', progress: 0.6, expiresLabel: '6m' },
  { id: 'c6', sender: 'Figma', type: 'code', value: '7X391P', label: '', progress: 0.85, expiresLabel: '8m' },
];

const CircularProgressAvatar = ({ name, progress, label }: { name: string; progress: number; label: string }) => {
  const metrics = getCircularAvatarMetrics();
  const circumference = 2 * Math.PI * metrics.radius;
  const strokeDashoffset = circumference - progress * circumference;
  const strokeColor = progress < 0.2 ? '#ef4444' : progress < 0.5 ? '#f59e0b' : '#3b82f6';

  return (
    <div className="circular-avatar-container">
      <svg width={metrics.outerSize} height={metrics.outerSize} className="circular-avatar-svg">
        <circle
          cx={metrics.center}
          cy={metrics.center}
          r={metrics.radius}
          fill="none"
          stroke="#e5e7eb"
          strokeWidth={metrics.strokeWidth}
        />
        <circle
          cx={metrics.center}
          cy={metrics.center}
          r={metrics.radius}
          fill="none"
          stroke={strokeColor}
          strokeWidth={metrics.strokeWidth}
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
        />
      </svg>
      <div className="circular-avatar-label" style={{ color: strokeColor }}>
        {label}
      </div>
      <div className="circular-avatar-icon">
        <Avatar name={name} size={metrics.innerSize} />
      </div>
    </div>
  );
};

const ExtractCard = ({ item, onRemove }: { item: ExtractItem; onRemove: () => void }) => {
  const [copied, setCopied] = useState(false);
  const [isTooltipVisible, setIsTooltipVisible] = useState(false);
  const [tooltipPosition, setTooltipPosition] = useState<{ left: number; top: number } | null>(null);
  const resetCopiedTimer = useRef<number | null>(null);
  const shellRef = useRef<HTMLDivElement>(null);
  const tooltipId = useId();

  useEffect(() => {
    return () => {
      if (resetCopiedTimer.current !== null) {
        window.clearTimeout(resetCopiedTimer.current);
      }
    };
  }, []);

  useEffect(() => {
    if (!isTooltipVisible) {
      return;
    }

    const syncTooltipPosition = () => {
      if (!shellRef.current) {
        return;
      }

      const bounds = shellRef.current.getBoundingClientRect();
      setTooltipPosition({
        left: bounds.left + bounds.width / 2,
        top: bounds.top - 14,
      });
    };

    syncTooltipPosition();
    window.addEventListener('resize', syncTooltipPosition);
    window.addEventListener('scroll', syncTooltipPosition, true);

    return () => {
      window.removeEventListener('resize', syncTooltipPosition);
      window.removeEventListener('scroll', syncTooltipPosition, true);
    };
  }, [isTooltipVisible]);

  const showTooltip = () => {
    if (shellRef.current) {
      const bounds = shellRef.current.getBoundingClientRect();
      setTooltipPosition({
        left: bounds.left + bounds.width / 2,
        top: bounds.top - 14,
      });
    }

    setIsTooltipVisible(true);
  };

  const hideTooltip = () => {
    setIsTooltipVisible(false);
  };

  const handleAction = () => {
    if (item.type === 'code') {
      void navigator.clipboard.writeText(item.value);
      setCopied(true);

      if (resetCopiedTimer.current !== null) {
        window.clearTimeout(resetCopiedTimer.current);
      }

      resetCopiedTimer.current = window.setTimeout(() => {
        setCopied(false);
        resetCopiedTimer.current = null;
      }, 2000);
      return;
    }

    window.open(item.value, '_blank');
  };

  const handleShellBlur = (event: FocusEvent<HTMLDivElement>) => {
    if (event.relatedTarget instanceof Node && event.currentTarget.contains(event.relatedTarget)) {
      return;
    }

    hideTooltip();
  };

  const handleClose = (event: MouseEvent<HTMLButtonElement>) => {
    event.stopPropagation();
    onRemove();
  };

  const handleCloseBlur = (event: FocusEvent<HTMLButtonElement>) => {
    if (event.relatedTarget instanceof Node && shellRef.current?.contains(event.relatedTarget)) {
      return;
    }

    hideTooltip();
  };

  const actionText = getExtractActionHint(item.type, copied);
  const tooltipContent = actionText === null ? null : item.type === 'link' ? (
    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
      <Open16Filled style={{ color: '#60a5fa' }} />
      <span>{actionText}</span>
    </div>
  ) : (
    <span>{actionText}</span>
  );

  return (
    <div
      ref={shellRef}
      className="extract-minimal-card-shell"
      onBlur={handleShellBlur}
      onFocus={showTooltip}
      onMouseEnter={showTooltip}
      onMouseLeave={hideTooltip}
    >
      <button
        aria-describedby={tooltipId}
        className="extract-minimal-card"
        type="button"
        onClick={handleAction}
      >
        <CircularProgressAvatar name={item.sender} progress={item.progress} label={item.expiresLabel} />
        <div
          className={`extract-minimal-code ${copied ? 'copied' : ''} ${item.type === 'link' ? 'link' : ''}`}
        >
          {copied ? 'Copied!' : (item.type === 'code' ? item.value : item.label)}
          {item.type === 'link' && !copied && <OpenRegular fontSize={14} style={{ marginLeft: 6 }} />}
        </div>
      </button>
      <ExtractTooltipPortal
        id={tooltipId}
        position={tooltipPosition}
        visible={isTooltipVisible && tooltipContent !== null}
      >
        {tooltipContent}
      </ExtractTooltipPortal>
      <button
        aria-label="Dismiss extract item"
        className="extract-minimal-close"
        type="button"
        onBlur={handleCloseBlur}
        onClick={handleClose}
        onFocus={showTooltip}
        onMouseEnter={showTooltip}
      >
        <DismissRegular fontSize={14} />
      </button>
    </div>
  );
};

export function MailWorkspace({ category }: { category: string }) {
  const [codes, setCodes] = useState<ExtractItem[]>(mockExtractedCodes);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [showLeft, setShowLeft] = useState(false);
  const [showRight, setShowRight] = useState(false);

  const checkScroll = () => {
    if (scrollRef.current) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current;
      setShowLeft(scrollLeft > 0);
      setShowRight(Math.ceil(scrollLeft + clientWidth) < scrollWidth);
    }
  };

  useEffect(() => {
    checkScroll();
    window.addEventListener('resize', checkScroll);
    return () => window.removeEventListener('resize', checkScroll);
  }, [codes]);

  const scroll = (direction: 'left' | 'right') => {
    if (scrollRef.current) {
      const amount = 300;
      scrollRef.current.scrollBy({ left: direction === 'left' ? -amount : amount, behavior: 'smooth' });
    }
  };

  return (
    <div className="workspace-content">
      {(category === 'inbox' || category === 'verifications') && codes.length > 0 ? (
        <div className="extract-zone-wrapper">
          {showLeft && (
            <div className="extract-scroll-button left" onClick={() => scroll('left')}>
              <ChevronLeftRegular fontSize={24} />
            </div>
          )}
          <div className="extract-zone" ref={scrollRef} onScroll={checkScroll}>
            {codes.map(item => (
              <ExtractCard
                key={item.id}
                item={item}
                onRemove={() => setCodes(prev => prev.filter(current => current.id !== item.id))}
              />
            ))}
          </div>
          {showRight && (
            <div className="extract-scroll-button right" onClick={() => scroll('right')}>
              <ChevronRightRegular fontSize={24} />
            </div>
          )}
        </div>
      ) : null}

      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
        <Text style={{ fontWeight: 600, fontSize: '18px' }}>
          {category === 'inbox' ? 'Inbox' : category === 'verifications' ? 'Verifications' : category === 'alerts' ? 'Security Alerts' : 'Archive'}
        </Text>

        <div className="mail-list">
          {mockMails.map((mail) => (
            <div key={mail.id} className={`mail-item ${mail.unread ? 'unread' : ''}`}>
              <div style={{ width: '140px', display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Avatar name={mail.sender} size={28} />
                <Text style={{ fontWeight: mail.unread ? 600 : 400 }}>{mail.sender}</Text>
              </div>

              <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '0 16px', overflow: 'hidden' }}>
                <Text style={{ fontWeight: mail.unread ? 600 : 400 }} className="text-truncate">{mail.subject}</Text>
                <Text style={{ color: '#6b7280', fontSize: '13px' }} className="text-truncate">{mail.preview}</Text>
              </div>

              <div style={{ width: '100px', textAlign: 'right', color: '#6b7280', fontSize: '13px' }}>
                {mail.time}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
