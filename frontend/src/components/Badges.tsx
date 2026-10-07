import type { TaskPriority, TaskStatus } from '../api/types';

const STATUS_LABEL: Record<TaskStatus, string> = {
  todo: 'To do',
  in_progress: 'In progress',
  done: 'Done',
};

export function PriorityBadge({ priority }: { priority: TaskPriority }) {
  return <span className={`badge priority-${priority}`}>{priority}</span>;
}

export function StatusBadge({ status }: { status: TaskStatus }) {
  return <span className={`badge status-${status}`}>{STATUS_LABEL[status]}</span>;
}

export function CacheBadge({ hit }: { hit: boolean }) {
  return (
    <span
      className={`badge ${hit ? 'cache-hit' : 'cache-miss'}`}
      title={hit ? 'Served from Redis cache' : 'Loaded from the database'}
    >
      cache.hit = {String(hit)}
    </span>
  );
}
