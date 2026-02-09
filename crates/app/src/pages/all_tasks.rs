use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::task_card::TaskCard;
use crate::pages::inbox::{
    complete_inbox_task, delete_inbox_task, update_inbox_task,
};

#[server(GetAllTasks, "/api")]
pub async fn get_all_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
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
        project_title: Option<String>,
        column_name: Option<String>,
        subtask_count: Option<i64>,
        tags: Option<serde_json::Value>,
    }

    let rows = sqlx::query_as::<_, TaskRow>(
        "SELECT t.id, t.project_id, t.parent_id, t.column_id, t.user_id, \
         t.title, t.body, t.position, t.sequential_limit, \
         t.start_date, t.due_date, t.completed_at, t.reviewed_at, \
         t.created_at, t.updated_at, \
         p.title as project_title, \
         pc.name as column_name, \
         (SELECT count(*) FROM tasks s WHERE s.parent_id = t.id) \
             as subtask_count, \
         (SELECT json_agg(tg.name) FROM task_tags tt \
          JOIN tags tg ON tg.id = tt.tag_id \
          WHERE tt.task_id = t.id) as tags \
         FROM tasks t \
         LEFT JOIN projects p ON p.id = t.project_id \
         LEFT JOIN project_columns pc ON pc.id = t.column_id \
         WHERE t.parent_id IS NULL \
           AND t.user_id = $1 \
         ORDER BY \
           CASE WHEN t.completed_at IS NULL THEN 0 ELSE 1 END, \
           t.position ASC, \
           t.completed_at DESC NULLS FIRST",
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
                task: north_domain::Task {
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
                project_title: row.project_title,
                column_name: row.column_name,
                tags,
                subtask_count: row.subtask_count.unwrap_or(0),
                actionable: row.completed_at.is_none(),
            }
        })
        .collect();

    Ok(tasks)
}

#[component]
pub fn AllTasksPage() -> impl IntoView {
    let all_tasks = Resource::new(|| (), |_| get_all_tasks());

    let update_action =
        Action::new(|input: &(i64, String, Option<String>)| {
            let (id, title, body) = input.clone();
            update_inbox_task(id, title, body)
        });

    let complete_action = Action::new(|id: &i64| {
        let id = *id;
        complete_inbox_task(id)
    });

    let delete_action = Action::new(|id: &i64| {
        let id = *id;
        delete_inbox_task(id)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = update_action.value().get() {
            all_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = complete_action.value().get() {
            all_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = delete_action.value().get() {
            all_tasks.refetch();
        }
    });

    let on_complete = move |id: i64| {
        complete_action.dispatch(id);
    };

    let on_delete = move |id: i64| {
        delete_action.dispatch(id);
    };

    let on_update =
        move |id: i64, title: String, body: Option<String>| {
            update_action.dispatch((id, title, body));
        };

    view! {
        <div class="space-y-4">
            <h1 class="text-xl font-semibold text-text-primary">
                "All Tasks"
            </h1>

            <Suspense fallback=move || {
                view! {
                    <div class="text-sm text-text-secondary py-4">
                        "Loading tasks..."
                    </div>
                }
            }>
                {move || {
                    let on_complete = on_complete.clone();
                    let on_delete = on_delete.clone();
                    let on_update = on_update.clone();
                    Suspend::new(async move {
                        match all_tasks.await {
                            Ok(tasks) => {
                                if tasks.is_empty() {
                                    view! {
                                        <div class="text-sm text-text-secondary \
                                                    py-8 text-center">
                                            "No tasks yet."
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div>
                                            {tasks
                                                .into_iter()
                                                .map(|task| {
                                                    let on_complete =
                                                        on_complete.clone();
                                                    let on_delete =
                                                        on_delete.clone();
                                                    let on_update =
                                                        on_update.clone();
                                                    view! {
                                                        <TaskCard
                                                            task=task
                                                            on_complete=on_complete
                                                            on_delete=on_delete
                                                            on_update=on_update
                                                        />
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    }
                                        .into_any()
                                }
                            }
                            Err(e) => {
                                view! {
                                    <div class="text-sm text-red-500 py-4">
                                        {format!(
                                            "Failed to load tasks: {e}"
                                        )}
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
