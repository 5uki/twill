import type { CSSProperties, ReactNode } from "react";
import { createPortal } from "react-dom";

interface ExtractTooltipPortalProps {
  children: ReactNode;
  id: string;
  position: {
    left: number;
    top: number;
  } | null;
  visible: boolean;
}

export function ExtractTooltipPortal({
  children,
  id,
  position,
  visible,
}: ExtractTooltipPortalProps) {
  if (typeof document === "undefined" || position === null) {
    return null;
  }

  const style = {
    left: `${position.left}px`,
    top: `${position.top}px`,
  } satisfies CSSProperties;

  return createPortal(
    <div
      aria-hidden={!visible}
      className="extract-floating-tooltip"
      data-visible={visible}
      id={id}
      role="tooltip"
      style={style}
    >
      {children}
    </div>,
    document.body,
  );
}
