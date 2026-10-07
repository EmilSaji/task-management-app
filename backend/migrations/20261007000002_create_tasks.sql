-- Tasks
CREATE TYPE task_status AS ENUM ('todo', 'in_progress', 'done');
-- Declared low -> high so that ORDER BY priority DESC yields high, medium, low.
CREATE TYPE task_priority AS ENUM ('low', 'medium', 'high');

CREATE TABLE tasks (
    id             UUID PRIMARY KEY,
    title          TEXT          NOT NULL CHECK (length(trim(title)) > 0),
    description    TEXT          NOT NULL DEFAULT '',
    status         task_status   NOT NULL DEFAULT 'todo',
    priority       task_priority NOT NULL DEFAULT 'medium',
    created_by_id  UUID          NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    assigned_to_id UUID          NULL     REFERENCES users (id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ   NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ   NOT NULL DEFAULT now()
);

CREATE INDEX idx_tasks_assigned_to_id ON tasks (assigned_to_id);

CREATE TRIGGER tasks_set_updated_at
    BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
