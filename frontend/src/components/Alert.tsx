import type { ReactNode } from 'react';

type Variant = 'error' | 'success' | 'info' | 'warning';

export function Alert({
  variant = 'info',
  title,
  children,
  onDismiss,
}: {
  variant?: Variant;
  title?: string;
  children: ReactNode;
  onDismiss?: () => void;
}) {
  return (
    <div className={`alert alert-${variant}`} role={variant === 'error' ? 'alert' : 'status'}>
      <div>
        {title && <strong className="alert-title">{title}</strong>}
        <div>{children}</div>
      </div>
      {onDismiss && (
        <button type="button" className="alert-close" aria-label="Dismiss" onClick={onDismiss}>
          ×
        </button>
      )}
    </div>
  );
}
