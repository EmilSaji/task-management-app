import { useState } from 'react';
import { tasksApi } from '../api/tasks';
import type { Task } from '../api/types';
import { errorMessage } from '../api/client';
import { Alert } from '../components/Alert';
import { AssignPanel } from '../components/AssignPanel';
import { Spinner } from '../components/Spinner';
import { TaskForm } from '../components/TaskForm';
import { TaskTable } from '../components/TaskTable';
import { useAsync } from '../hooks/useAsync';

export function AdminPage() {
  const tasks = useAsync<Task[]>(tasksApi.list);
  const [selected, setSelected] = useState<Set<string>>(new Set());

  const toggle = (id: string) =>
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });

  const toggleAll = (checked: boolean) =>
    setSelected(checked ? new Set(tasks.data?.map((t) => t.id)) : new Set());

  const onAssigned = () => {
    setSelected(new Set());
    void tasks.reload();
  };

  const total = tasks.data?.length ?? 0;
  const assigned = tasks.data?.filter((t) => t.assigned_to).length ?? 0;

  return (
    <div className="grid">
      <section className="card">
        <h2>Create task</h2>
        <TaskForm onCreated={() => void tasks.reload()} />
      </section>

      <section className="card">
        <div className="section-head">
          <h2>All tasks</h2>
          <span className="muted">
            {total} total · {assigned} assigned
          </span>
          <button type="button" className="btn btn-ghost" onClick={() => void tasks.reload()}>
            Refresh
          </button>
        </div>

        <AssignPanel selectedIds={[...selected]} onAssigned={onAssigned} />

        {tasks.loading && !tasks.data ? (
          <Spinner label="Loading tasks…" />
        ) : tasks.error ? (
          <Alert variant="error">{errorMessage(tasks.error)}</Alert>
        ) : (
          <TaskTable
            tasks={tasks.data ?? []}
            selected={selected}
            onToggle={toggle}
            onToggleAll={toggleAll}
          />
        )}
      </section>
    </div>
  );
}
