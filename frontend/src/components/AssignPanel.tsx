import { useEffect, useState } from 'react';
import { errorMessage } from '../api/client';
import { tasksApi } from '../api/tasks';
import { usersApi } from '../api/users';
import type { User } from '../api/types';
import { useAsync } from '../hooks/useAsync';
import { Alert } from './Alert';
import { Spinner } from './Spinner';

interface Props {
  selectedIds: string[];
  onAssigned: () => void;
}

/** Pick a staff member and assign the selected tasks to them. */
export function AssignPanel({ selectedIds, onAssigned }: Props) {
  const staff = useAsync<User[]>(() => usersApi.list('staff'));
  const [assignee, setAssignee] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<{ ok: boolean; message: string } | null>(null);

  // Default to the first staff member (James Bond in the validation flow).
  useEffect(() => {
    if (!assignee && staff.data?.length) setAssignee(staff.data[0].email);
  }, [assignee, staff.data]);

  const assign = async () => {
    setSubmitting(true);
    setResult(null);
    try {
      const res = await tasksApi.assign(selectedIds, assignee);
      setResult({ ok: true, message: `Assigned ${res.assigned_count} task(s) to ${res.assigned_to}.` });
      onAssigned();
    } catch (err) {
      setResult({ ok: false, message: errorMessage(err) });
    } finally {
      setSubmitting(false);
    }
  };

  if (staff.loading) return <Spinner label="Loading staff…" />;
  if (staff.error) return <Alert variant="error">{errorMessage(staff.error)}</Alert>;
  if (!staff.data?.length) return <Alert variant="info">No staff users exist yet.</Alert>;

  return (
    <div className="assign-panel">
      {result && (
        <Alert variant={result.ok ? 'success' : 'error'} onDismiss={() => setResult(null)}>
          {result.message}
        </Alert>
      )}
      <div className="row">
        <label className="inline">
          Assign to
          <select value={assignee} onChange={(e) => setAssignee(e.target.value)}>
            {staff.data.map((u) => (
              <option key={u.id} value={u.email}>
                {u.full_name} ({u.email})
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className="btn btn-primary"
          disabled={submitting || selectedIds.length === 0 || !assignee}
          onClick={assign}
        >
          {submitting ? 'Assigning…' : `Assign ${selectedIds.length} selected`}
        </button>
      </div>
    </div>
  );
}
