import { useState } from 'react';
import type { FormEvent } from 'react';
import { ApiError, errorMessage } from '../api/client';
import { tasksApi } from '../api/tasks';
import type { Task, TaskPriority } from '../api/types';
import { Alert } from './Alert';

interface Props {
  onCreated?: (task: Task) => void;
}

/** Create-task form. Shows a dedicated message when the API answers 403. */
export function TaskForm({ onCreated }: Props) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<TaskPriority>('medium');
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<
    { kind: 'success' | 'error' | 'forbidden'; message: string } | null
  >(null);

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setResult(null);
    try {
      const task = await tasksApi.create({ title: title.trim(), description, priority });
      setResult({ kind: 'success', message: `Task “${task.title}” created.` });
      setTitle('');
      setDescription('');
      onCreated?.(task);
    } catch (err) {
      if (err instanceof ApiError && err.isForbidden) {
        setResult({
          kind: 'forbidden',
          message: `You are not allowed to create tasks — only admins can. (403 Forbidden: ${err.message})`,
        });
      } else {
        setResult({ kind: 'error', message: errorMessage(err) });
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form className="form" onSubmit={submit} noValidate>
      {result?.kind === 'success' && (
        <Alert variant="success" onDismiss={() => setResult(null)}>
          {result.message}
        </Alert>
      )}
      {result?.kind === 'forbidden' && (
        <Alert variant="warning" title="Access denied">
          {result.message}
        </Alert>
      )}
      {result?.kind === 'error' && <Alert variant="error">{result.message}</Alert>}

      <label>
        Title
        <input value={title} maxLength={200} onChange={(e) => setTitle(e.target.value)} required />
      </label>
      <label>
        Description
        <textarea
          rows={2}
          maxLength={2000}
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />
      </label>
      <label>
        Priority
        <select value={priority} onChange={(e) => setPriority(e.target.value as TaskPriority)}>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
        </select>
      </label>
      <button className="btn btn-primary" type="submit" disabled={submitting || !title.trim()}>
        {submitting ? 'Creating…' : 'Create task'}
      </button>
    </form>
  );
}
