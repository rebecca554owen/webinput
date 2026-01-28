use crate::config::Config;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub all_ips: Arc<Mutex<Vec<String>>>,
    pub selected_ip: Arc<Mutex<String>>,
    pub main_ip: Arc<Mutex<String>>,
    pub access_url: Arc<Mutex<String>>,
    pub port: Arc<Mutex<String>>,
    pub is_running: Arc<Mutex<bool>>,
    pub server_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub auto_paste: Arc<Mutex<bool>>,
    pub auto_enter: Arc<Mutex<bool>>,
}

impl AppState {
    pub fn new() -> Self {
        let config = Config::load();
        let port = config.port.clone();
        let was_running = config.was_running;
        let auto_paste = config.auto_paste;
        let auto_enter = config.auto_enter;

        let all_ips = get_all_ips();
        let main_ip = all_ips.get(1).cloned().unwrap_or_else(|| "127.0.0.1".to_string());
        let selected_ip = main_ip.clone();
        let access_url = format!("http://{}:{}", selected_ip, port);

        Self {
            config: Arc::new(Mutex::new(config)),
            all_ips: Arc::new(Mutex::new(all_ips)),
            selected_ip: Arc::new(Mutex::new(selected_ip)),
            main_ip: Arc::new(Mutex::new(main_ip)),
            access_url: Arc::new(Mutex::new(access_url)),
            port: Arc::new(Mutex::new(port)),
            is_running: Arc::new(Mutex::new(was_running)),
            server_handle: Arc::new(Mutex::new(None)),
            auto_paste: Arc::new(Mutex::new(auto_paste)),
            auto_enter: Arc::new(Mutex::new(auto_enter)),
        }
    }
}

fn get_all_ips() -> Vec<String> {
    let mut ips = Vec::new();

    match local_ip_address::local_ip() {
        Ok(ip) => {
            ips.push(ip.to_string());
        }
        Err(_) => {
            ips.push("127.0.0.1".to_string());
        }
    }

    if let Ok(ifas) = local_ip_address::list_afinet_netifas() {
        for (_name, addr) in ifas {
            if addr.is_ipv4() && !addr.is_loopback() && !(matches!(addr, std::net::IpAddr::V4(ip4) if ip4.is_link_local())) {
                let ip_str = addr.to_string();
                if !ips.contains(&ip_str) {
                    ips.push(ip_str);
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }

    let mut priority192 = Vec::new();
    let mut priority10 = Vec::new();
    let mut other_ips = Vec::new();
    let mut virtual_ips = Vec::new();

    for ip in &ips {
        if ip.starts_with("198.18.") {
            virtual_ips.push(ip.clone());
        } else if ip.starts_with("192.168.") {
            priority192.push(ip.clone());
        } else if ip.starts_with("10.") {
            priority10.push(ip.clone());
        } else if ip.starts_with("172.") {
            let parts: Vec<&str> = ip.split('.').collect();
            if parts.len() >= 2 {
                if let Ok(second) = parts[1].parse::<u8>() {
                    if second >= 16 && second <= 31 {
                        virtual_ips.push(ip.clone());
                    } else {
                        other_ips.push(ip.clone());
                    }
                } else {
                    other_ips.push(ip.clone());
                }
            } else {
                other_ips.push(ip.clone());
            }
        } else {
            other_ips.push(ip.clone());
        }
    }

    let mut result = priority192;
    result.extend(priority10);
    result.extend(other_ips);
    result.extend(virtual_ips);

    result.insert(0, "0.0.0.0".to_string());

    result
}
