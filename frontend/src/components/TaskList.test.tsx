import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ApiError } from '../api/client';
import type { MyTask } from '../api/types';
import { TaskList } from './TaskList';

const tasks: MyTask[] = [
  { id: '1', title: 'Infiltrate SPECTRE meeting', status: 'todo', priority: 'high', assigned_to: 'jamesbond@example.com' },
  { id: '2', title: 'Collect gadgets from Q', status: 'todo', priority: 'medium', assigned_to: 'jamesbond@example.com' },
  { id: '3', title: 'Brief M on findings', status: 'todo', priority: 'low', assigned_to: 'jamesbond@example.com' },
];

describe('TaskList', () => {
  it('shows a loading state', () => {
    render(<TaskList tasks={null} loading error={null} />);
    expect(screen.getByRole('status')).toHaveTextContent('Loading your tasks');
  });

  it('shows an error state', () => {
    render(<TaskList tasks={null} loading={false} error={new ApiError(500, 'internal_error', 'Boom')} />);
    expect(screen.getByRole('alert')).toHaveTextContent('Boom');
  });

  it('shows an empty state', () => {
    render(<TaskList tasks={[]} loading={false} error={null} />);
    expect(screen.getByText('No tasks assigned to you')).toBeInTheDocument();
  });

  it('renders the tasks returned by the API', () => {
    render(<TaskList tasks={tasks} loading={false} error={null} />);
    const items = screen.getAllByRole('listitem');
    expect(items).toHaveLength(3);
    expect(items[0]).toHaveTextContent('Infiltrate SPECTRE meeting');
    expect(items[0]).toHaveTextContent('high');
  });
});
