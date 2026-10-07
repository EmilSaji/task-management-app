import { Navigate, Route, Routes } from 'react-router-dom';
import { Layout } from './components/Layout';
import { ProtectedRoute } from './components/ProtectedRoute';
import { homePathFor, useAuth } from './context/AuthContext';
import { AdminPage } from './pages/AdminPage';
import { LoginPage } from './pages/LoginPage';
import { StaffPage } from './pages/StaffPage';

export function App() {
  const { user } = useAuth();

  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        element={
          <ProtectedRoute>
            <Layout />
          </ProtectedRoute>
        }
      >
        <Route
          path="/admin"
          element={
            <ProtectedRoute role="admin">
              <AdminPage />
            </ProtectedRoute>
          }
        />
        <Route path="/my-tasks" element={<StaffPage />} />
      </Route>
      <Route path="*" element={<Navigate to={user ? homePathFor(user) : '/login'} replace />} />
    </Routes>
  );
}
