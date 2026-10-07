use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::models::{Role, TaskPriority, TaskStatus, TaskWithUsers};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 200, message = "must be 1-200 characters"))]
    #[schema(example = "Infiltrate SPECTRE meeting")]
    pub title: String,
    #[validate(length(max = 2000, message = "must be at most 2000 characters"))]
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

/// Partial update. Admins may change any field; the assignee may change only `status`.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 200, message = "must be 1-200 characters"))]
    pub title: Option<String>,
    #[validate(length(max = 2000, message = "must be at most 2000 characters"))]
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
}

impl UpdateTaskRequest {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.status.is_none()
            && self.priority.is_none()
    }

    pub fn only_status(&self) -> bool {
        self.title.is_none() && self.description.is_none() && self.priority.is_none()
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignTasksRequest {
    #[validate(length(min = 1, max = 100, message = "must contain 1-100 task ids"))]
    pub task_ids: Vec<Uuid>,
    #[validate(email(message = "must be a valid email address"))]
    #[schema(example = "jamesbond@example.com")]
    pub assignee_email: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssignTasksResponse {
    pub assigned_to: String,
    pub task_ids: Vec<Uuid>,
    pub assigned_count: usize,
}

/// Full task view (admin endpoints).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_by: String,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TaskWithUsers> for TaskResponse {
    fn from(row: TaskWithUsers) -> Self {
        let t = row.task;
        Self {
            id: t.id,
            title: t.title,
            description: t.description,
            status: t.status,
            priority: t.priority,
            created_by: row.created_by_email,
            assigned_to: row.assigned_to_email,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

// ---- GET /tasks/view-my-tasks ----------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MyTasksUser {
    pub email: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MyTaskItem {
    pub id: Uuid,
    pub title: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assigned_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskSummary {
    pub total_assigned_tasks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CacheInfo {
    /// `false` when the response was built from the database, `true` when served from Redis.
    pub hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MyTasksResponse {
    pub user: MyTasksUser,
    pub tasks: Vec<MyTaskItem>,
    pub summary: TaskSummary,
    pub cache: CacheInfo,
}
