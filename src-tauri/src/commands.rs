use crate::app::AppState;
use tauri::State;

/// 获取所有可用的 IP 地址列表
#[tauri::command]
pub async fn get_ips(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.all_ips.lock().await.clone())
}

/// 获取当前选中的 IP 地址
#[tauri::command]
pub async fn get_selected_ip(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.selected_ip.lock().await.clone())
}

/// 获取主 IP 地址
#[tauri::command]
pub async fn get_main_ip(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.main_ip.lock().await.clone())
}

/// 获取当前端口号
#[tauri::command]
pub async fn get_port(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.port.lock().await.clone())
}

/// 设置选中的 IP 地址
#[tauri::command]
pub async fn set_selected_ip(ip: String, state: State<'_, AppState>) -> Result<(), String> {
    *state.selected_ip.lock().await = ip.clone();
    *state.access_url.lock().await = format!("http://{}:{}", ip, *state.port.lock().await);
    Ok(())
}

/// 设置端口号
///
/// 如果服务器正在运行，会先停止服务器，更新端口后再重启
#[tauri::command]
pub async fn set_port(port: String, state: State<'_, AppState>) -> Result<(), String> {
    if !port.chars().all(|c| c.is_ascii_digit()) {
        return Err("端口必须为数字".to_string());
    }

    let was_running = *state.is_running.lock().await;
    if was_running {
        stop_server(state.clone()).await?;
    }

    let port_clone = port.clone();
    state.update_config(|config| {
        config.port = port_clone;
    }).await?;

    *state.port.lock().await = port;

    if was_running {
        start_server(state.clone()).await?;
    }

    Ok(())
}

/// 启动 WebSocket 服务器
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

    state.update_config(|config| {
        config.was_running = true;
    }).await?;

    *state.is_running.lock().await = true;

    Ok(())
}

/// 停止 WebSocket 服务器
#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    *state.is_running.lock().await = false;

    state.update_config(|config| {
        config.was_running = false;
    }).await?;

    if let Some(handle) = state.server_handle.lock().await.take() {
        handle.abort();
    }

    Ok(())
}

/// 获取访问 URL
#[tauri::command]
pub async fn get_access_url(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.access_url.lock().await.clone())
}

/// 检查服务器是否正在运行
#[tauri::command]
pub async fn is_running(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.is_running.lock().await)
}

/// 获取自动粘贴设置
#[tauri::command]
pub async fn get_auto_paste(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_paste.lock().await)
}

/// 设置自动粘贴
#[tauri::command]
pub async fn set_auto_paste(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    *state.auto_paste.lock().await = enabled;

    state.update_config(|config| {
        config.auto_paste = enabled;
    }).await?;

    Ok(())
}

/// 获取自动回车设置
#[tauri::command]
pub async fn get_auto_enter(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_enter.lock().await)
}

/// 设置自动回车
#[tauri::command]
pub async fn set_auto_enter(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    *state.auto_enter.lock().await = enabled;

    state.update_config(|config| {
        config.auto_enter = enabled;
    }).await?;

    Ok(())
}
