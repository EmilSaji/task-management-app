import type { MyTask } from '../api/types';
import { errorMessage } from '../api/client';
import { Alert } from './Alert';
import { PriorityBadge, StatusBadge } from './Badges';
import { EmptyState } from './EmptyState';
import { Spinner } from './Spinner';

interface Props {
  tasks: MyTask[] | null;
  loading: boolean;
  error: unknown;
}

/** The logged-in user's assigned tasks, with loading / error / empty states. */
export function TaskList({ tasks, loading, error }: Props) {
  if (loading && !tasks) return <Spinner label="Loading your tasks…" />;
  if (error) return <Alert variant="error">{errorMessage(error)}</Alert>;
  if (!tasks || tasks.length === 0) {
    return <EmptyState title="No tasks assigned to you">Check back once an admin assigns work.</EmptyState>;
  }

  return (
    <ul className="task-list" aria-label="My tasks">
      {tasks.map((task) => (
        <li key={task.id} className="task-item">
          <div>
            <div className="task-title">{task.title}</div>
            <div className="muted small">
              {task.assigned_to} · <code>{task.id}</code>
            </div>
          </div>
          <div className="task-badges">
            <PriorityBadge priority={task.priority} />
            <StatusBadge status={task.status} />
          </div>
        </li>
      ))}
    </ul>
  );
}
