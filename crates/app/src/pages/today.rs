use leptos::prelude::*;
use north_domain::TaskWithMeta;

use crate::components::task_card::TaskCard;
use crate::pages::inbox::{
    complete_inbox_task, delete_inbox_task, uncomplete_task,
    update_inbox_task,
};

#[server(GetTodayTasks, "/api")]
pub async fn get_today_tasks() -> Result<Vec<TaskWithMeta>, ServerFnError> {
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
        start_at: Option<chrono::DateTime<chrono::Utc>>,
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
         t.start_at, t.due_date, t.completed_at, t.reviewed_at, \
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
         WHERE t.user_id = $1 \
           AND t.start_at IS NOT NULL \
           AND t.start_at::date <= CURRENT_DATE \
           AND t.completed_at IS NULL \
         ORDER BY t.start_at ASC",
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
                    start_at: row.start_at,
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
                actionable: true,
            }
        })
        .collect();

    Ok(tasks)
}

#[server(SetTaskStartAt, "/api")]
pub async fn set_task_start_at(
    id: i64,
    start_at: String,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let dt = chrono::NaiveDateTime::parse_from_str(
        &start_at,
        "%Y-%m-%dT%H:%M",
    )
    .or_else(|_| {
        chrono::NaiveDateTime::parse_from_str(
            &start_at,
            "%Y-%m-%dT%H:%M:%S",
        )
    })
    .map_err(|e| ServerFnError::new(format!("Invalid datetime: {e}")))?;

    let dt_utc = dt.and_utc();

    let result = sqlx::query(
        "UPDATE tasks SET start_at = $1 \
         WHERE id = $2 AND user_id = $3",
    )
    .bind(dt_utc)
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[server(ClearTaskStartAt, "/api")]
pub async fn clear_task_start_at(id: i64) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();
    let user_id = crate::server_fns::auth::get_auth_user_id().await?;

    let result = sqlx::query(
        "UPDATE tasks SET start_at = NULL \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Task not found".to_string()));
    }

    Ok(())
}

#[component]
pub fn TodayPage() -> impl IntoView {
    let today_tasks = Resource::new(|| (), |_| get_today_tasks());

    let update_action =
        Action::new(|input: &(i64, String, Option<String>)| {
            let (id, title, body) = input.clone();
            update_inbox_task(id, title, body)
        });

    let complete_action = Action::new(|id: &i64| {
        let id = *id;
        complete_inbox_task(id)
    });

    let uncomplete_action = Action::new(|id: &i64| {
        let id = *id;
        uncomplete_task(id)
    });

    let delete_action = Action::new(|id: &i64| {
        let id = *id;
        delete_inbox_task(id)
    });

    let set_start_at_action =
        Action::new(|input: &(i64, String)| {
            let (id, start_at) = input.clone();
            set_task_start_at(id, start_at)
        });

    let clear_start_at_action = Action::new(|id: &i64| {
        let id = *id;
        clear_task_start_at(id)
    });

    Effect::new(move || {
        if let Some(Ok(_)) = update_action.value().get() {
            today_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = delete_action.value().get() {
            today_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = complete_action.value().get() {
            today_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = uncomplete_action.value().get() {
            today_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = set_start_at_action.value().get() {
            today_tasks.refetch();
        }
    });

    Effect::new(move || {
        if let Some(Ok(_)) = clear_start_at_action.value().get() {
            today_tasks.refetch();
        }
    });

    let on_toggle_complete = move |id: i64, was_completed: bool| {
        if was_completed {
            uncomplete_action.dispatch(id);
        } else {
            complete_action.dispatch(id);
        }
    };

    let on_delete = move |id: i64| {
        delete_action.dispatch(id);
    };

    let on_update =
        move |id: i64, title: String, body: Option<String>| {
            update_action.dispatch((id, title, body));
        };

    let on_set_start_at =
        Callback::new(move |(id, start_at): (i64, String)| {
            set_start_at_action.dispatch((id, start_at));
        });

    let on_clear_start_at = Callback::new(move |id: i64| {
        clear_start_at_action.dispatch(id);
    });

    view! {
        <div class="space-y-4">
            <h1 class="text-xl font-semibold text-text-primary">"Today"</h1>

            <Suspense fallback=move || {
                view! {
                    <div class="text-sm text-text-secondary py-4">
                        "Loading tasks..."
                    </div>
                }
            }>
                {move || {
                    let on_toggle_complete = on_toggle_complete.clone();
                    let on_delete = on_delete.clone();
                    let on_update = on_update.clone();
                    Suspend::new(async move {
                        match today_tasks.await {
                            Ok(tasks) => {
                                if tasks.is_empty() {
                                    view! {
                                        <div class="text-sm text-text-secondary \
                                                    py-8 text-center">
                                            "No tasks scheduled for today."
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div>
                                            {tasks
                                                .into_iter()
                                                .map(|task| {
                                                    let on_toggle_complete =
                                                        on_toggle_complete
                                                            .clone();
                                                    let on_delete =
                                                        on_delete.clone();
                                                    let on_update =
                                                        on_update.clone();
                                                    view! {
                                                        <TaskCard
                                                            task=task
                                                            on_toggle_complete=on_toggle_complete
                                                            on_delete=on_delete
                                                            on_update=on_update
                                                            on_set_start_at=on_set_start_at
                                                            on_clear_start_at=on_clear_start_at
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
