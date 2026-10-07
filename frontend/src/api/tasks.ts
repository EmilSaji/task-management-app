import { apiRequest } from './client';
import type {
  AssignTasksResponse,
  CreateTaskInput,
  MyTasksResponse,
  Task,
  TaskStatus,
} from './types';

export const tasksApi = {
  list: () => apiRequest<Task[]>('/tasks'),

  create: (input: CreateTaskInput) =>
    apiRequest<Task>('/tasks', { method: 'POST', body: input }),

  assign: (taskIds: string[], assigneeEmail: string) =>
    apiRequest<AssignTasksResponse>('/tasks/assign', {
      method: 'POST',
      body: { task_ids: taskIds, assignee_email: assigneeEmail },
    }),

  updateStatus: (taskId: string, status: TaskStatus) =>
    apiRequest<Task>(`/tasks/${taskId}`, { method: 'PATCH', body: { status } }),

  viewMine: () => apiRequest<MyTasksResponse>('/tasks/view-my-tasks'),
};
