import { clsx } from 'clsx';

export function StatusDot({ ok }: { ok: boolean }) {
  return (
    <span
      className={clsx(
        'inline-block w-2.5 h-2.5 rounded-full shrink-0',
        ok ? 'bg-emerald-500' : 'bg-red-500',
      )}
    />
  );
}
