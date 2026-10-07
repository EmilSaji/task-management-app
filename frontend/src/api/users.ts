import { apiRequest } from './client';
import type { Role, User } from './types';

export const usersApi = {
  list: (role?: Role) => apiRequest<User[]>(role ? `/users?role=${role}` : '/users'),
};
