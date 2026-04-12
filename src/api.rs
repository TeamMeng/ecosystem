use crate::{
    model::{CreateTaskRequest, Task, TaskStatus},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn create_task(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut id_guard = state.next_id.lock().await;
    let task_id = *id_guard;
    *id_guard += 1;
    drop(id_guard);

    let task = Task {
        id: task_id,
        payload: req.payload,
        status: TaskStatus::Pending,
        retry_count: 0,
    };

    {
        let mut tasks_guard = state.tasks.lock().await;
        tasks_guard.insert(task_id, task.clone());
    }

    state
        .task_tx
        .send(task_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn get_task(
    Path(task_id): Path<u64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let tasks_guard = state.tasks.lock().await;
    let task = tasks_guard
        .get(&task_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok((StatusCode::OK, Json(task)))
}
