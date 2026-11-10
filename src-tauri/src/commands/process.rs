use pctoolkit_platform::{
    close_processes, get_top_resource_processes, probe_running_processes, terminate_process_by_pid,
    ProcessActionLog, ProcessCloseBatchResult, RunningProcessGroup, TopProcessInfo,
};
use serde::Deserialize;

use crate::commands::error::{run_blocking, CommandResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessNamesRequest {
    pub names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloseProcessesRequest {
    pub names: Vec<String>,
    pub force: bool,
}

#[tauri::command]
pub async fn probe_running_processes_command(
    request: ProcessNamesRequest,
) -> CommandResult<Vec<RunningProcessGroup>> {
    run_blocking("probe_running_processes", move || {
        probe_running_processes(&request.names).map_err(pctoolkit_core::CoreError::from)
    })
    .await
}

#[tauri::command]
pub async fn close_running_processes_command(
    request: CloseProcessesRequest,
) -> CommandResult<ProcessCloseBatchResult> {
    run_blocking("close_running_processes", move || {
        close_processes(&request.names, request.force).map_err(pctoolkit_core::CoreError::from)
    })
    .await
}

#[tauri::command]
pub async fn get_top_resource_processes_command(limit: Option<usize>) -> CommandResult<Vec<TopProcessInfo>> {
    run_blocking("get_top_resource_processes", move || {
        Ok::<_, pctoolkit_core::CoreError>(get_top_resource_processes(limit.unwrap_or(10)))
    })
    .await
}

#[tauri::command]
pub async fn terminate_process_command(pid: u32) -> CommandResult<ProcessActionLog> {
    run_blocking("terminate_process", move || {
        Ok::<_, pctoolkit_core::CoreError>(terminate_process_by_pid(pid))
    })
    .await
}
