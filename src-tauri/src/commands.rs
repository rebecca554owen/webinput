use crate::app::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_ips(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.all_ips.lock().await.clone())
}

#[tauri::command]
pub async fn get_selected_ip(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.selected_ip.lock().await.clone())
}

#[tauri::command]
pub async fn get_main_ip(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.main_ip.lock().await.clone())
}

#[tauri::command]
pub async fn get_port(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.port.lock().await.clone())
}

#[tauri::command]
pub async fn set_selected_ip(ip: String, state: State<'_, AppState>) -> Result<(), String> {
    *state.selected_ip.lock().await = ip.clone();
    *state.access_url.lock().await = format!("http://{}:{}", ip, *state.port.lock().await);
    Ok(())
}

#[tauri::command]
pub async fn set_port(port: String, state: State<'_, AppState>) -> Result<(), String> {
    if !port.chars().all(|c| c.is_ascii_digit()) {
        return Err("端口必须为数字".to_string());
    }

    let was_running = *state.is_running.lock().await;
    if was_running {
        stop_server(state.clone()).await?;
    }

    *state.port.lock().await = port.clone();
    state.config.lock().await.port = port.clone();
    state.config.lock().await.save().map_err(|e| e.to_string())?;

    if was_running {
        start_server(state.clone()).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn start_server(state: State<'_, AppState>) -> Result<(), String> {
    let is_running = *state.is_running.lock().await;
    if is_running {
        return Err("服务器已在运行".to_string());
    }

    let config = state.config.lock().await.clone();
    let selected_ip = state.selected_ip.lock().await.clone();
    let port = state.port.lock().await.clone();
    let access_url = format!("http://{}:{}", selected_ip, port);
    let auto_enter = state.auto_enter.clone();

    let handle = tokio::spawn(async move {
        let server = crate::server::Server::new(config, auto_enter);
        let _ = server.start().await;
    });

    *state.server_handle.lock().await = Some(handle);
    *state.access_url.lock().await = access_url.clone();

    state.config.lock().await.was_running = true;
    state.config.lock().await.save().map_err(|e| e.to_string())?;

    *state.is_running.lock().await = true;

    Ok(())
}

#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    *state.is_running.lock().await = false;

    state.config.lock().await.was_running = false;
    state.config.lock().await.save().map_err(|e| e.to_string())?;

    if let Some(handle) = state.server_handle.lock().await.take() {
        handle.abort();
    }

    Ok(())
}

#[tauri::command]
pub async fn get_access_url(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.access_url.lock().await.clone())
}

#[tauri::command]
pub async fn is_running(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.is_running.lock().await)
}

#[tauri::command]
pub async fn get_auto_paste(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_paste.lock().await)
}

#[tauri::command]
pub async fn set_auto_paste(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    *state.auto_paste.lock().await = enabled;
    state.config.lock().await.auto_paste = enabled;
    state.config.lock().await.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_auto_enter(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_enter.lock().await)
}

#[tauri::command]
pub async fn set_auto_enter(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    *state.auto_enter.lock().await = enabled;
    state.config.lock().await.auto_enter = enabled;
    state.config.lock().await.save().map_err(|e| e.to_string())?;
    Ok(())
}
