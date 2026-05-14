use crate::presentation::print_header;
use crate::storage;
use crate::models::Server;
use anyhow::Result;
use owo_colors::OwoColorize;

pub fn handle_suggest(name: String, command: Option<String>) -> Result<()> {
    let store = storage::load_store()?;

    let server = match storage::get_server(&store, &name) {
        Some(s) => s,
        None => {
            anyhow::bail!("Server '{}' not found", name);
        }
    };

    print_suggestions(server, command.as_deref());

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
        ("disk", "Disk Usage", format!("{} 'df -h'", ssh)),
        ("disk_inode", "Inode Usage", format!("{} 'df -i'", ssh)),
        (
            "disk_large",
            "Find Large Directories",
            format!("{} 'du -sh /* 2>/dev/null | sort -hr | head -20'", ssh),
        ),
        (
            "cpu",
            "CPU Info & Load",
            format!("{} 'cat /proc/cpuinfo | grep processor | wc -l && uptime'", ssh),
        ),
        ("memory", "Memory Usage", format!("{} 'free -h'", ssh)),
        (
            "proc_mem",
            "Top Memory Processes",
            format!("{} 'ps aux --sort=-%mem | head -10'", ssh),
        ),
        (
            "proc_cpu",
            "Top CPU Processes",
            format!("{} 'ps aux --sort=-%cpu | head -10'", ssh),
        ),
        ("port", "Port Usage (ss)", format!("{} 'ss -tlnp'", ssh)),
        (
            "port_netstat",
            "Port Usage (netstat)",
            format!("{} 'netstat -tlnp'", ssh),
        ),
        (
            "listen_port",
            "Find Process on Port",
            format!("{} 'lsof -i :{}'", ssh, port),
        ),
        (
            "connection",
            "Network Connections",
            format!("{} 'ss -tan'", ssh),
        ),
        (
            "sys_info",
            "System Information",
            format!("{} 'uname -a && cat /etc/os-release'", ssh),
        ),
        ("uptime", "System Uptime", format!("{} 'uptime'", ssh)),
        ("who", "Logged In Users", format!("{} 'who'", ssh)),
        ("last", "Recent Logins", format!("{} 'last -10'", ssh)),
        (
            "service",
            "Running Services",
            format!("{} 'systemctl list-units --type=service --state=running'", ssh),
        ),
        (
            "service_status",
            "Check Service Status",
            format!("{} 'systemctl status nginx'", ssh),
        ),
        (
            "journal",
            "Systemd Journal",
            format!("{} 'journalctl -xe --no-pager -n 50'", ssh),
        ),
        ("docker_ps", "Docker Containers", format!("{} 'docker ps'", ssh)),
        (
            "docker_psa",
            "All Docker Containers",
            format!("{} 'docker ps -a'", ssh),
        ),
        (
            "docker_images",
            "Docker Images",
            format!("{} 'docker images'", ssh),
        ),
        (
            "docker_logs",
            "Docker Container Logs",
            format!("{} 'docker logs --tail 100 container_name'", ssh),
        ),
        (
            "docker_stats",
            "Docker Stats",
            format!("{} 'docker stats --no-stream'", ssh),
        ),
        (
            "docker_cleanup",
            "Docker Cleanup",
            format!("{} 'docker system prune -af'", ssh),
        ),
        (
            "nginx_access",
            "Nginx Access Log",
            format!("{} 'tail -100 /var/log/nginx/access.log'", ssh),
        ),
        (
            "nginx_error",
            "Nginx Error Log",
            format!("{} 'tail -100 /var/log/nginx/error.log'", ssh),
        ),
        ("syslog", "System Logs", format!("{} 'tail -100 /var/log/syslog'", ssh)),
        (
            "auth_log",
            "Auth Logs (Failed Login)",
            format!("{} 'grep failed /var/log/auth.log | tail -50'", ssh),
        ),
        ("cron", "Cron Jobs", format!("{} 'crontab -l'", ssh)),
        ("sysctl", "Kernel Parameters", format!("{} 'sysctl -a'", ssh)),
        ("limits", "User Limits", format!("{} 'ulimit -a'", ssh)),
        (
            "firewall",
            "Firewall Status (ufw)",
            format!("{} 'ufw status'", ssh),
        ),
        (
            "firewall_iptables",
            "iptables Rules",
            format!("{} 'iptables -L -n'", ssh),
        ),
        ("mount", "Mount Points", format!("{} 'mount | column -t'", ssh)),
        ("fstab", "Fstab Config", format!("{} 'cat /etc/fstab'", ssh)),
        ("dns", "DNS Configuration", format!("{} 'cat /etc/resolv.conf'", ssh)),
        ("hosts", "Hosts File", format!("{} 'cat /etc/hosts'", ssh)),
        (
            "process_tree",
            "Process Tree",
            format!("{} 'pstree -p'", ssh),
        ),
        (
            "killed_procs",
            "OOM Killed Processes",
            format!("{} 'dmesg | grep -i killed | tail -20'", ssh),
        ),
        (
            "sysload",
            "System Load (vmstat)",
            format!("{} 'vmstat 1 5'", ssh),
        ),
        ("iostat", "IO Statistics", format!("{} 'iostat -xz 1 5'", ssh)),
        (
            "mpstat",
            "CPU Per-Core Stats",
            format!("{} 'mpstat -P ALL 1 1'", ssh),
        ),
        (
            "sar_net",
            "Network Stats (sar)",
            format!("{} 'sar -n DEV 1 3'", ssh),
        ),
        (
            "tcpdump",
            "Capture Traffic (sudo)",
            format!("{} 'sudo tcpdump -i eth0 -c 100'", ssh),
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
