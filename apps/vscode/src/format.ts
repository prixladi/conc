import { ProjectInfo, ServiceInfo } from './client';

export function formatDuration(seconds: number): string {
  seconds = Math.max(0, Math.floor(seconds));
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (d > 0) {
    return `${d}d ${h}h`;
  }
  if (h > 0) {
    return `${h}h ${m}m`;
  }
  if (m > 0) {
    return `${m}m ${s}s`;
  }
  return `${s}s`;
}

export function nowSeconds(): number {
  return Math.floor(Date.now() / 1000);
}

/** Short human description of a service's current state, e.g. "up 3m 2s". */
export function describeService(s: ServiceInfo, now = nowSeconds()): string {
  switch (s.status) {
    case 'RUNNING':
      return s.startTime ? `up ${formatDuration(now - s.startTime)}` : 'running';
    case 'STOPPED':
      return s.stopTime ? `stopped ${formatDuration(now - s.stopTime)} ago` : 'stopped';
    case 'EXITED':
      return 'exited';
    default:
      return 'idle';
  }
}

export function runningCount(p: ProjectInfo): number {
  return p.services.filter((s) => s.status === 'RUNNING').length;
}

export function formatTimestamp(seconds: number): string {
  return seconds ? new Date(seconds * 1000).toLocaleString() : '-';
}
