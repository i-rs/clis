use clap::{Parser, Subcommand};
use keyring::use_native_store;
use keyring_core::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const SERVICE_NAME: &str = "i-rs-server";

fn init_keyring() {
    let _ = use_native_store(false);
}

#[derive(Parser, Debug)]
#[command(name = "i-rs-server")]
#[command(about = "Server management CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "HOST")]
        host: String,
        #[arg(value_name = "PORT")]
        port: Option<u16>,
        #[arg(short, long)]
        user: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        #[arg(short, long)]
        note: Vec<String>,
    },
    Delete {
        #[arg(value_name = "NAME")]
        name: String,
    },
    List {
        #[arg(short, long)]
        tag: Option<String>,
    },
    Update {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(long)]
        host: Option<String>,
        #[arg(short = 'P', long)]
        port: Option<u16>,
        #[arg(short, long)]
        user: Option<String>,
        #[arg(short, long)]
        password: Option<String>,
        #[arg(short, long)]
        tag: Option<Vec<String>>,
        #[arg(short, long)]
        note: Option<Vec<String>>,
    },
    Get {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Suggest {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(short, long)]
        command: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(skip)]
    pub password: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStore {
    pub servers: HashMap<String, Server>,
}

impl Default for ServerStore {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }
}

fn get_data_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("CONFIG_DIR") {
        PathBuf::from(config_dir).join("i-rs").join("servers.json")
    } else {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        config_dir.join("i-rs").join("servers.json")
    }
}

fn store_password(server_name: &str, password: &str) -> anyhow::Result<()> {
    let entry = Entry::new(SERVICE_NAME, server_name)
        .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;
    entry.set_password(password)
        .map_err(|e| anyhow::anyhow!("Failed to store password: {}", e))?;
    Ok(())
}

fn get_password(server_name: &str) -> anyhow::Result<Option<String>> {
    match Entry::new(SERVICE_NAME, server_name) {
        Ok(entry) => match entry.get_password() {
            Ok(pwd) => Ok(Some(pwd)),
            Err(e) => {
                let err_str = format!("{}", e);
                if err_str.contains("NoEntry") || err_str.contains("not found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Failed to get password: {}", e))
                }
            }
        },
        Err(_) => Ok(None),
    }
}

fn delete_password(server_name: &str) -> anyhow::Result<()> {
    match Entry::new(SERVICE_NAME, server_name) {
        Ok(entry) => {
            let _ = entry.delete_credential();
        }
        Err(_) => {}
    }
    Ok(())
}

fn load_store() -> anyhow::Result<ServerStore> {
    let path = get_data_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(ServerStore::default())
    }
}

fn save_store(store: &ServerStore) -> anyhow::Result<()> {
    let path = get_data_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

fn ssh_cmd(user: &Option<String>, host: &str, port: u16) -> String {
    match user {
        Some(u) => format!("ssh {}@{} -p {}", u, host, port),
        None => format!("ssh root@{} -p {}", host, port),
    }
}

fn print_suggestions(server: &Server, filter: Option<&str>) {
    let user = &server.user;
    let host = &server.host;
    let port = server.port;
    let ssh = ssh_cmd(user, host, port);

    println!("\n=== Server: {} ({}@{}:{}) ===\n", server.name, user.as_deref().unwrap_or("-"), host, port);
    println!("# SSH Connection");
    println!("  {}\n", ssh);

    let filter_str = filter.unwrap_or("").to_lowercase();

    let all_commands = vec![
        ("ssh", "SSH Login", ssh.clone()),
        ("ssh_key", "SSH Key Setup (Passwordless)", format!("ssh-keygen -t rsa -b 4096 -C '{}@{}' -f ~/.ssh/id_rsa -N '' && ssh-copy-id {}@{}", user.as_deref().unwrap_or("root"), host, user.as_deref().unwrap_or("root"), host)),
        ("scp_up", "SCP Upload File", format!("scp -P {} local-file.txt {}@{}:/tmp/", port, user.as_deref().unwrap_or("root"), host)),
        ("scp_down", "SCP Download File", format!("scp -P {} {}@{}:/tmp/remote-file.txt ./", port, user.as_deref().unwrap_or("root"), host)),
        ("disk", "Disk Usage", format!("{} 'df -h'", ssh)),
        ("disk_inode", "Inode Usage", format!("{} 'df -i'", ssh)),
        ("disk_large", "Find Large Directories", format!("{} 'du -sh /* 2>/dev/null | sort -hr | head -20'", ssh)),
        ("cpu", "CPU Info & Load", format!("{} 'cat /proc/cpuinfo | grep processor | wc -l && uptime'", ssh)),
        ("memory", "Memory Usage", format!("{} 'free -h'", ssh)),
        ("proc_mem", "Top Memory Processes", format!("{} 'ps aux --sort=-%mem | head -10'", ssh)),
        ("proc_cpu", "Top CPU Processes", format!("{} 'ps aux --sort=-%cpu | head -10'", ssh)),
        ("port", "Port Usage (ss)", format!("{} 'ss -tlnp'", ssh)),
        ("port_netstat", "Port Usage (netstat)", format!("{} 'netstat -tlnp'", ssh)),
        ("listen_port", "Find Process on Port", format!("{} 'lsof -i :{}'", ssh, port)),
        ("connection", "Network Connections", format!("{} 'ss -tan'", ssh)),
        ("sys_info", "System Information", format!("{} 'uname -a && cat /etc/os-release'", ssh)),
        ("uptime", "System Uptime", format!("{} 'uptime'", ssh)),
        ("who", "Logged In Users", format!("{} 'who'", ssh)),
        ("last", "Recent Logins", format!("{} 'last -10'", ssh)),
        ("service", "Running Services", format!("{} 'systemctl list-units --type=service --state=running'", ssh)),
        ("service_status", "Check Service Status", format!("{} 'systemctl status nginx'", ssh)),
        ("journal", "Systemd Journal", format!("{} 'journalctl -xe --no-pager -n 50'", ssh)),
        ("docker_ps", "Docker Containers", format!("{} 'docker ps'", ssh)),
        ("docker_psa", "All Docker Containers", format!("{} 'docker ps -a'", ssh)),
        ("docker_images", "Docker Images", format!("{} 'docker images'", ssh)),
        ("docker_logs", "Docker Container Logs", format!("{} 'docker logs --tail 100 container_name'", ssh)),
        ("docker_stats", "Docker Stats", format!("{} 'docker stats --no-stream'", ssh)),
        ("docker_cleanup", "Docker Cleanup", format!("{} 'docker system prune -af'", ssh)),
        ("nginx_access", "Nginx Access Log", format!("{} 'tail -100 /var/log/nginx/access.log'", ssh)),
        ("nginx_error", "Nginx Error Log", format!("{} 'tail -100 /var/log/nginx/error.log'", ssh)),
        ("syslog", "System Logs", format!("{} 'tail -100 /var/log/syslog'", ssh)),
        ("auth_log", "Auth Logs (Failed Login)", format!("{} 'grep failed /var/log/auth.log | tail -50'", ssh)),
        ("cron", "Cron Jobs", format!("{} 'crontab -l'", ssh)),
        ("sysctl", "Kernel Parameters", format!("{} 'sysctl -a'", ssh)),
        ("limits", "User Limits", format!("{} 'ulimit -a'", ssh)),
        ("firewall", "Firewall Status (ufw)", format!("{} 'ufw status'", ssh)),
        ("firewall_iptables", "iptables Rules", format!("{} 'iptables -L -n'", ssh)),
        ("mount", "Mount Points", format!("{} 'mount | column -t'", ssh)),
        ("fstab", "Fstab Config", format!("{} 'cat /etc/fstab'", ssh)),
        ("dns", "DNS Configuration", format!("{} 'cat /etc/resolv.conf'", ssh)),
        ("hosts", "Hosts File", format!("{} 'cat /etc/hosts'", ssh)),
        ("process_tree", "Process Tree", format!("{} 'pstree -p'", ssh)),
        ("killed_procs", "OOM Killed Processes", format!("{} 'dmesg | grep -i killed | tail -20'", ssh)),
        ("sysload", "System Load (vmstat)", format!("{} 'vmstat 1 5'", ssh)),
        ("iostat", "IO Statistics", format!("{} 'iostat -xz 1 5'", ssh)),
        ("mpstat", "CPU Per-Core Stats", format!("{} 'mpstat -P ALL 1 1'", ssh)),
        ("sar_net", "Network Stats (sar)", format!("{} 'sar -n DEV 1 3'", ssh)),
        ("tcpdump", "Capture Traffic (sudo)", format!("{} 'sudo tcpdump -i eth0 -c 100'", ssh)),
    ];

    if filter_str.is_empty() {
        println!("# File Transfer");
        println!("  # Upload:  scp -P {} local-file.txt {}@{}:/tmp/", port, user.as_deref().unwrap_or("root"), host);
        println!("  # Download: scp -P {} {}@{}:/tmp/file .\n", port, user.as_deref().unwrap_or("root"), host);
        println!("# System Commands");
        for (cmd, desc, _) in &all_commands[2..] {
            println!("  {:15} - {}", cmd, desc);
        }
    } else {
        for (cmd, desc, full_cmd) in &all_commands {
            if cmd.contains(&filter_str) || desc.to_lowercase().contains(&filter_str) {
                println!("# {} - {}", cmd, desc);
                println!("  {}\n", full_cmd);
            }
        }
    }

    println!("\nTips:");
    println!("  - Run with filter: i-rs-server suggest {} --command disk", server.name);
}

fn main() -> anyhow::Result<()> {
    init_keyring();
    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            name,
            host,
            port,
            user,
            password,
            tag,
            note,
        } => {
            let store = load_store()?;
            if store.servers.contains_key(&name) {
                anyhow::bail!("Server '{}' already exists", name);
            }
            let port = port.unwrap_or(22);
            if let Some(ref pwd) = password {
                store_password(&name, pwd)?;
            }
            let server = Server {
                name: name.clone(),
                host,
                port,
                user,
                password: None,
                tags: tag,
                notes: note,
            };
            let mut store = store;
            store.servers.insert(name.clone(), server);
            save_store(&store)?;
            println!("Server '{}' added successfully", name);
            if password.is_some() {
                println!("Password stored securely in keychain");
            }
        }
        Commands::Delete { name } => {
            let mut store = load_store()?;
            if store.servers.remove(&name).is_none() {
                anyhow::bail!("Server '{}' not found", name);
            }
            delete_password(&name)?;
            save_store(&store)?;
            println!("Server '{}' deleted successfully", name);
        }
        Commands::List { tag } => {
            let store = load_store()?;
            let servers: Vec<&Server> = if let Some(tag) = tag {
                store
                    .servers
                    .values()
                    .filter(|s| s.tags.contains(&tag))
                    .collect()
            } else {
                store.servers.values().collect()
            };

            if servers.is_empty() {
                println!("No servers found.");
            } else {
                for server in servers {
                    println!(
                        "{}  {}@{}:{}  tags: [{}]",
                        server.name,
                        server.user.as_deref().unwrap_or("-"),
                        server.host,
                        server.port,
                        server.tags.join(", ")
                    );
                }
            }
        }
        Commands::Update {
            name,
            host,
            port,
            user,
            password,
            tag,
            note,
        } => {
            let mut store = load_store()?;
            let server = store.servers.get_mut(&name);
            if server.is_none() {
                anyhow::bail!("Server '{}' not found", name);
            }
            let server = server.unwrap();
            if let Some(host) = host {
                server.host = host;
            }
            if let Some(port) = port {
                server.port = port;
            }
            if let Some(user) = user {
                server.user = Some(user);
            }
            if let Some(password) = password {
                store_password(&name, &password)?;
                println!("Password updated and stored securely in keychain");
            }
            if let Some(tag) = tag {
                server.tags = tag;
            }
            if let Some(note) = note {
                server.notes = note;
            }
            save_store(&store)?;
            println!("Server '{}' updated successfully", name);
        }
        Commands::Get { name } => {
            let store = load_store()?;
            let server = store.servers.get(&name);
            match server {
                Some(s) => {
                    println!("Name: {}", s.name);
                    println!("Host: {}", s.host);
                    println!("Port: {}", s.port);
                    if let Some(ref user) = s.user {
                        println!("User: {}", user);
                    }
                    if let Ok(Some(_)) = get_password(&name) {
                        println!("Password: (stored securely in keychain)");
                    }
                    if !s.tags.is_empty() {
                        println!("Tags: [{}]", s.tags.join(", "));
                    }
                    if !s.notes.is_empty() {
                        println!("Notes: {}", s.notes.join("; "));
                    }
                }
                None => anyhow::bail!("Server '{}' not found", name),
            }
        }
        Commands::Suggest { name, command } => {
            let store = load_store()?;
            let server = store.servers.get(&name);
            match server {
                Some(s) => {
                    print_suggestions(s, command.as_deref());
                }
                None => anyhow::bail!("Server '{}' not found", name),
            }
        }
    }

    Ok(())
}
