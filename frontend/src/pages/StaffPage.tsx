import { tasksApi } from '../api/tasks';
import type { MyTasksResponse } from '../api/types';
import { CacheBadge } from '../components/Badges';
import { TaskForm } from '../components/TaskForm';
import { TaskList } from '../components/TaskList';
import { useAuth } from '../context/AuthContext';
import { useAsync } from '../hooks/useAsync';

export function StaffPage() {
  const { isAdmin } = useAuth();
  const mine = useAsync<MyTasksResponse>(tasksApi.viewMine);

  return (
    <div className={isAdmin ? 'grid single' : 'grid wide-first'}>
      <section className="card">
        <div className="section-head">
          <h2>My tasks</h2>
          {mine.data && (
            <>
              <span className="muted">
                {mine.data.summary.total_assigned_tasks} assigned to {mine.data.user.email}
              </span>
              <CacheBadge hit={mine.data.cache.hit} />
            </>
          )}
          <button
            type="button"
            className="btn btn-ghost"
            disabled={mine.loading}
            onClick={() => void mine.reload()}
          >
            {mine.loading && mine.data ? 'Refreshing…' : 'Refresh'}
          </button>
        </div>
        <TaskList tasks={mine.data?.tasks ?? null} loading={mine.loading} error={mine.error} />
      </section>

      {!isAdmin && (
        <section className="card">
          <h2>Create task</h2>
          <p className="muted small">
            Staff accounts can't create tasks. Submitting shows how the API's 403 response is
            handled.
          </p>
          <TaskForm />
        </section>
      )}
    </div>
  );
}
