import type { Task } from '../api/types';
import { PriorityBadge, StatusBadge } from './Badges';
import { EmptyState } from './EmptyState';

interface Props {
  tasks: Task[];
  selected: Set<string>;
  onToggle: (id: string) => void;
  onToggleAll: (checked: boolean) => void;
}

/** Admin view of all tasks with row selection for assignment. */
export function TaskTable({ tasks, selected, onToggle, onToggleAll }: Props) {
  if (tasks.length === 0) {
    return <EmptyState title="No tasks yet">Create the first task using the form.</EmptyState>;
  }
  const allSelected = tasks.every((t) => selected.has(t.id));

  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>
              <input
                type="checkbox"
                aria-label="Select all tasks"
                checked={allSelected}
                onChange={(e) => onToggleAll(e.target.checked)}
              />
            </th>
            <th>Title</th>
            <th>Priority</th>
            <th>Status</th>
            <th>Assigned to</th>
          </tr>
        </thead>
        <tbody>
          {tasks.map((task) => (
            <tr key={task.id} className={selected.has(task.id) ? 'selected' : undefined}>
              <td>
                <input
                  type="checkbox"
                  aria-label={`Select ${task.title}`}
                  checked={selected.has(task.id)}
                  onChange={() => onToggle(task.id)}
                />
              </td>
              <td>
                <div>{task.title}</div>
                {task.description && <div className="muted small">{task.description}</div>}
              </td>
              <td>
                <PriorityBadge priority={task.priority} />
              </td>
              <td>
                <StatusBadge status={task.status} />
              </td>
              <td>{task.assigned_to ?? <span className="muted">Unassigned</span>}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
