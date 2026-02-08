use leptos::prelude::*;
use north_domain::{Task, TaskWithMeta};

use crate::components::task_card::TaskCard;
use crate::components::task_form::InlineTaskForm;

#[server(GetInboxTasks, "/api")]
pub async fn get_inbox_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    #[derive(sqlx::FromRow)]
    struct TaskRow {
        id: i64,
        project_id: Option<i64>,
        parent_id: Option<i64>,
        column_id: Option<i64>,
        user_id: i64,
        title: String,
        body: Option<String>,
        position: i32,
        sequential_limit: i16,
        start_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        subtask_count: Option<i64>,
        tags: Option<serde_json::Value>,
    }

    let rows = sqlx::query_as::<_, TaskRow>(
        "SELECT t.id, t.project_id, t.parent_id, t.column_id, t.user_id, \
         t.title, t.body, t.position, t.sequential_limit, \
         t.start_date, t.due_date, t.completed_at, t.reviewed_at, \
         t.created_at, t.updated_at, \
         (SELECT count(*) FROM tasks s WHERE s.parent_id = t.id) as subtask_count, \
         (SELECT json_agg(tg.name) FROM task_tags tt \
          JOIN tags tg ON tg.id = tt.tag_id WHERE tt.task_id = t.id) as tags \
         FROM tasks t \
         WHERE t.project_id IS NULL \
           AND t.parent_id IS NULL \
           AND t.user_id = $1 \
           AND t.completed_at IS NULL \
         ORDER BY t.position",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let tasks = rows
        .into_iter()
        .map(|row| {
            let tags: Vec<String> = row
                .tags
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            TaskWithMeta {
                task: Task {
                    id: row.id,
                    project_id: row.project_id,
                    parent_id: row.parent_id,
                    column_id: row.column_id,
                    user_id: row.user_id,
                    title: row.title,
                    body: row.body,
                    position: row.position,
                    sequential_limit: row.sequential_limit,
                    start_date: row.start_date,
                    due_date: row.due_date,
                    completed_at: row.completed_at,
                    reviewed_at: row.reviewed_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
                project_title: None,
                column_name: None,
                tags,
                subtask_count: row.subtask_count.unwrap_or(0),
                actionable: true,
            }
        })
        .collect();

    Ok(tasks)
}

#[server(CreateInboxTask, "/api")]
pub async fn create_inbox_task(
    title: String,
) -> Result<Task, ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let max_pos: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(position) FROM tasks \
         WHERE user_id = $1 AND project_id IS NULL AND parent_id IS NULL",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let position = max_pos.unwrap_or(0) + 1;

    #[derive(sqlx::FromRow)]
    struct InsertedTask {
        id: i64,
        project_id: Option<i64>,
        parent_id: Option<i64>,
        column_id: Option<i64>,
        user_id: i64,
        title: String,
        body: Option<String>,
        position: i32,
        sequential_limit: i16,
        start_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let row = sqlx::query_as::<_, InsertedTask>(
        "INSERT INTO tasks (user_id, title, position, sequential_limit) \
         VALUES ($1, $2, $3, 1) \
         RETURNING id, project_id, parent_id, column_id, user_id, \
         title, body, position, sequential_limit, \
         start_date, due_date, completed_at, reviewed_at, \
         created_at, updated_at",
    )
    .bind(user_id)
    .bind(&title)
    .bind(position)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(Task {
        id: row.id,
        project_id: row.project_id,
        parent_id: row.parent_id,
        column_id: row.column_id,
        user_id: row.user_id,
        title: row.title,
        body: row.body,
        position: row.position,
        sequential_limit: row.sequential_limit,
        start_date: row.start_date,
        due_date: row.due_date,
        completed_at: row.completed_at,
        reviewed_at: row.reviewed_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

#[component]
pub fn InboxPage() -> impl IntoView {
    let inbox_tasks = Resource::new(|| (), |_| get_inbox_tasks());

    let create_action = Action::new(|title: &String| {
        let title = title.clone();
        create_inbox_task(title)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = create_action.value().get() {
            inbox_tasks.refetch();
        }
    });

    let on_create = move |title: String| {
        create_action.dispatch(title);
    };

    view! {
        <div class="space-y-4">
            <h1 class="text-xl font-semibold text-teal-950">"Inbox"</h1>

            <InlineTaskForm on_submit=on_create/>

            <Suspense fallback=move || {
                view! {
                    <div class="text-sm text-sage-400 py-4">"Loading tasks..."</div>
                }
            }>
                {move || Suspend::new(async move {
                    match inbox_tasks.await {
                        Ok(tasks) => {
                            if tasks.is_empty() {
                                view! {
                                    <div class="text-sm text-sage-400 py-8 text-center">
                                        "No tasks in your inbox. Add one above."
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="space-y-2">
                                        {tasks
                                            .into_iter()
                                            .map(|task| view! { <TaskCard task=task/> })
                                            .collect::<Vec<_>>()}
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                        Err(e) => {
                            view! {
                                <div class="text-sm text-red-500 py-4">
                                    {format!("Failed to load tasks: {e}")}
                                </div>
                            }
                                .into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
