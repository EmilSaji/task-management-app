// Types mirroring the Rust API DTOs (backend/src/dto).

export type Role = 'admin' | 'staff';
export type TaskStatus = 'todo' | 'in_progress' | 'done';
export type TaskPriority = 'low' | 'medium' | 'high';

export interface User {
  id: string;
  full_name: string;
  email: string;
  role: Role;
  created_at: string;
}

export interface LoginResponse {
  login_challenge_id: string;
  two_factor_required: boolean;
  expires_at: string;
  message: string;
}

export interface TokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: TaskPriority;
  created_by: string;
  assigned_to: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTaskInput {
  title: string;
  description?: string;
  priority?: TaskPriority;
  status?: TaskStatus;
}

export interface AssignTasksResponse {
  assigned_to: string;
  task_ids: string[];
  assigned_count: number;
}

export interface MyTask {
  id: string;
  title: string;
  status: TaskStatus;
  priority: TaskPriority;
  assigned_to: string;
}

export interface MyTasksResponse {
  user: { email: string; role: Role };
  tasks: MyTask[];
  summary: { total_assigned_tasks: number };
  cache: { hit: boolean };
}

export interface DevEmail {
  to: string;
  subject: string;
  body: string;
  code: string;
  login_challenge_id: string;
  sent_at: string;
}

export interface ApiErrorBody {
  error: { code: string; message: string };
}
