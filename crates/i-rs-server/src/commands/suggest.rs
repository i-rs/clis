use crate::models::Server;
use crate::presentation::print_header;
use crate::storage;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_suggest(name: String, command: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let server = match store.get_entry(&name) {
        Some(s) => s,
        None => {
            anyhow::bail!("Server '{name}' not found");
        }
    };

    print_suggestions(server, command.as_deref());

    Ok(())
}

fn ssh_cmd(user: &Option<String>, host: &str, port: u16) -> String {
    match user {
        Some(u) => format!("ssh {u}@{host} -p {port}"),
        None => format!("ssh root@{host} -p {port}"),
    }
}

fn print_suggestions(server: &Server, filter: Option<&str>) {
    let user = &server.user;
    let host = &server.host;
    let port = server.port;
    let ssh = ssh_cmd(user, host, port);

    print_header(&format!(
        "Server: {} ({}@{}:{})",
        server.name.green(),
        user.as_deref().unwrap_or("-").yellow(),
        host.cyan(),
        port.to_string().cyan()
    ));

    println!("\n{} {}\n", "SSH:".bold(), ssh.cyan());

    let filter_str = filter.unwrap_or("").to_lowercase();

    let all_commands = vec![
        ("ssh", "SSH Login", ssh.clone()),
        (
            "ssh_key",
            "SSH Key Setup (Passwordless)",
            format!(
                "ssh-keygen -t rsa -b 4096 -C '{}@{}' -f ~/.ssh/id_rsa -N '' && ssh-copy-id {}@{}",
                user.as_deref().unwrap_or("root"),
                host,
                user.as_deref().unwrap_or("root"),
                host
            ),
        ),
        (
            "scp_up",
            "SCP Upload File",
            format!(
                "scp -P {} local-file.txt {}@{}:/tmp/",
                port,
                user.as_deref().unwrap_or("root"),
                host
            ),
        ),
        (
            "scp_down",
            "SCP Download File",
            format!(
                "scp -P {} {}@{}:/tmp/remote-file.txt ./",
                port,
                user.as_deref().unwrap_or("root"),
                host
            ),
        ),
        ("disk", "Disk Usage", format!("{ssh} 'df -h'")),
        ("disk_inode", "Inode Usage", format!("{ssh} 'df -i'")),
        (
            "disk_large",
            "Find Large Directories",
            format!("{ssh} 'du -sh /* 2>/dev/null | sort -hr | head -20'"),
        ),
        (
            "cpu",
            "CPU Info & Load",
            format!("{ssh} 'cat /proc/cpuinfo | grep processor | wc -l && uptime'"),
        ),
        ("memory", "Memory Usage", format!("{ssh} 'free -h'")),
        (
            "proc_mem",
            "Top Memory Processes",
            format!("{ssh} 'ps aux --sort=-%mem | head -10'"),
        ),
        (
            "proc_cpu",
            "Top CPU Processes",
            format!("{ssh} 'ps aux --sort=-%cpu | head -10'"),
        ),
        ("port", "Port Usage (ss)", format!("{ssh} 'ss -tlnp'")),
        (
            "port_netstat",
            "Port Usage (netstat)",
            format!("{ssh} 'netstat -tlnp'"),
        ),
        (
            "listen_port",
            "Find Process on Port",
            format!("{ssh} 'lsof -i :{port}'"),
        ),
        (
            "connection",
            "Network Connections",
            format!("{ssh} 'ss -tan'"),
        ),
        (
            "sys_info",
            "System Information",
            format!("{ssh} 'uname -a && cat /etc/os-release'"),
        ),
        ("uptime", "System Uptime", format!("{ssh} 'uptime'")),
        ("who", "Logged In Users", format!("{ssh} 'who'")),
        ("last", "Recent Logins", format!("{ssh} 'last -10'")),
        (
            "service",
            "Running Services",
            format!("{ssh} 'systemctl list-units --type=service --state=running'"),
        ),
        (
            "service_status",
            "Check Service Status",
            format!("{ssh} 'systemctl status nginx'"),
        ),
        (
            "journal",
            "Systemd Journal",
            format!("{ssh} 'journalctl -xe --no-pager -n 50'"),
        ),
        (
            "docker_ps",
            "Docker Containers",
            format!("{ssh} 'docker ps'"),
        ),
        (
            "docker_psa",
            "All Docker Containers",
            format!("{ssh} 'docker ps -a'"),
        ),
        (
            "docker_images",
            "Docker Images",
            format!("{ssh} 'docker images'"),
        ),
        (
            "docker_logs",
            "Docker Container Logs",
            format!("{ssh} 'docker logs --tail 100 container_name'"),
        ),
        (
            "docker_stats",
            "Docker Stats",
            format!("{ssh} 'docker stats --no-stream'"),
        ),
        (
            "docker_cleanup",
            "Docker Cleanup",
            format!("{ssh} 'docker system prune -af'"),
        ),
        (
            "nginx_access",
            "Nginx Access Log",
            format!("{ssh} 'tail -100 /var/log/nginx/access.log'"),
        ),
        (
            "nginx_error",
            "Nginx Error Log",
            format!("{ssh} 'tail -100 /var/log/nginx/error.log'"),
        ),
        (
            "syslog",
            "System Logs",
            format!("{ssh} 'tail -100 /var/log/syslog'"),
        ),
        (
            "auth_log",
            "Auth Logs (Failed Login)",
            format!("{ssh} 'grep failed /var/log/auth.log | tail -50'"),
        ),
        ("cron", "Cron Jobs", format!("{ssh} 'crontab -l'")),
        ("sysctl", "Kernel Parameters", format!("{ssh} 'sysctl -a'")),
        ("limits", "User Limits", format!("{ssh} 'ulimit -a'")),
        (
            "firewall",
            "Firewall Status (ufw)",
            format!("{ssh} 'ufw status'"),
        ),
        (
            "firewall_iptables",
            "iptables Rules",
            format!("{ssh} 'iptables -L -n'"),
        ),
        (
            "mount",
            "Mount Points",
            format!("{ssh} 'mount | column -t'"),
        ),
        ("fstab", "Fstab Config", format!("{ssh} 'cat /etc/fstab'")),
        (
            "dns",
            "DNS Configuration",
            format!("{ssh} 'cat /etc/resolv.conf'"),
        ),
        ("hosts", "Hosts File", format!("{ssh} 'cat /etc/hosts'")),
        ("process_tree", "Process Tree", format!("{ssh} 'pstree -p'")),
        (
            "killed_procs",
            "OOM Killed Processes",
            format!("{ssh} 'dmesg | grep -i killed | tail -20'"),
        ),
        (
            "sysload",
            "System Load (vmstat)",
            format!("{ssh} 'vmstat 1 5'"),
        ),
        ("iostat", "IO Statistics", format!("{ssh} 'iostat -xz 1 5'")),
        (
            "mpstat",
            "CPU Per-Core Stats",
            format!("{ssh} 'mpstat -P ALL 1 1'"),
        ),
        (
            "sar_net",
            "Network Stats (sar)",
            format!("{ssh} 'sar -n DEV 1 3'"),
        ),
        (
            "tcpdump",
            "Capture Traffic (sudo)",
            format!("{ssh} 'sudo tcpdump -i eth0 -c 100'"),
        ),
    ];

    if filter_str.is_empty() {
        println!("{}", "System Commands:".bold().yellow());
        for (cmd, desc, _) in &all_commands[2..] {
            println!("  {:18} {}", cmd.magenta(), desc);
        }
    } else {
        for (cmd, desc, full_cmd) in &all_commands {
            if cmd.contains(&filter_str) || desc.to_lowercase().contains(&filter_str) {
                println!(
                    "{} {}\n  {}\n",
                    cmd.magenta().bold(),
                    "-".dimmed(),
                    full_cmd.cyan()
                );
            }
        }
    }

    println!(
        "\n{} Use '{}' to filter commands",
        "Tip:".dimmed(),
        "--command <filter>".cyan()
    );
}
