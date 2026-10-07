import { NavLink, Outlet, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

export function Layout() {
  const { user, isAdmin, signOut } = useAuth();
  const navigate = useNavigate();

  const logout = () => {
    signOut();
    navigate('/login', { replace: true });
  };

  return (
    <div className="app">
      <header className="topbar">
        <span className="brand">Task Manager</span>
        <nav>
          {isAdmin && <NavLink to="/admin">Admin</NavLink>}
          <NavLink to="/my-tasks">My tasks</NavLink>
        </nav>
        {user && (
          <div className="user-chip">
            <span>
              {user.full_name} <span className={`badge role-${user.role}`}>{user.role}</span>
            </span>
            <button type="button" className="btn btn-ghost" onClick={logout}>
              Log out
            </button>
          </div>
        )}
      </header>
      <main className="container">
        <Outlet />
      </main>
    </div>
  );
}
