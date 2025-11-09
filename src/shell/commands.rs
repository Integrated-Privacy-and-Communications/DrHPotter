//! Command implementations for the fake shell

use std::collections::HashMap;
use std::path::PathBuf;

use super::filesystem::FakeFilesystem;

/// Execute a command in the fake shell
pub async fn execute_command(
    cmd: &str,
    args: &[&str],
    filesystem: &mut FakeFilesystem,
    current_dir: &mut PathBuf,
    env_vars: &HashMap<String, String>,
) -> String {
    match cmd {
        "pwd" => cmd_pwd(current_dir),
        "whoami" => cmd_whoami(env_vars),
        "id" => cmd_id(),
        "uname" => cmd_uname(args),
        "ls" => cmd_ls(args, filesystem, current_dir),
        "cd" => cmd_cd(args, current_dir, filesystem),
        "cat" => cmd_cat(args, filesystem, current_dir),
        "echo" => cmd_echo(args, env_vars),
        "env" => cmd_env(env_vars),
        "ps" => cmd_ps(),
        "ifconfig" => cmd_ifconfig(),
        "ip" => cmd_ip(args),
        "netstat" => cmd_netstat(),
        "wget" => cmd_wget(args).await,
        "curl" => cmd_curl(args).await,
        "chmod" => cmd_chmod(args),
        "chown" => cmd_chown(args),
        "rm" => cmd_rm(args),
        "mkdir" => cmd_mkdir(args),
        "touch" => cmd_touch(args),
        "cp" => cmd_cp(args),
        "mv" => cmd_mv(args),
        "history" => cmd_history(),
        "exit" | "logout" => cmd_exit(),
        _ => format!("{}: command not found\n", cmd),
    }
}

fn cmd_pwd(current_dir: &PathBuf) -> String {
    format!("{}\n", current_dir.display())
}

fn cmd_whoami(env_vars: &HashMap<String, String>) -> String {
    let user = env_vars.get("USER").map(|s| s.as_str()).unwrap_or("root");
    format!("{}\n", user)
}

fn cmd_id() -> String {
    "uid=0(root) gid=0(root) groups=0(root)\n".to_string()
}

fn cmd_uname(args: &[&str]) -> String {
    if args.contains(&"-a") {
        "Linux honeypot 5.15.0-58-generic #64-Ubuntu SMP Thu Jan 5 11:43:13 UTC 2023 x86_64 x86_64 x86_64 GNU/Linux\n".to_string()
    } else if args.contains(&"-r") {
        "5.15.0-58-generic\n".to_string()
    } else if args.contains(&"-s") {
        "Linux\n".to_string()
    } else if args.contains(&"-n") {
        "honeypot\n".to_string()
    } else if args.contains(&"-m") {
        "x86_64\n".to_string()
    } else {
        "Linux\n".to_string()
    }
}

fn cmd_ls(args: &[&str], filesystem: &FakeFilesystem, current_dir: &PathBuf) -> String {
    let show_hidden = args.contains(&"-a") || args.contains(&"-la") || args.contains(&"-al");
    let long_format = args.contains(&"-l") || args.contains(&"-la") || args.contains(&"-al");

    let entries = filesystem.list_dir(current_dir);

    if long_format {
        let mut output = String::new();
        for entry in entries {
            if !show_hidden && entry.starts_with('.') {
                continue;
            }
            output.push_str(&format!(
                "drwxr-xr-x 2 root root 4096 Nov  9 10:30 {}\n",
                entry
            ));
        }
        output
    } else {
        let filtered: Vec<_> = entries
            .into_iter()
            .filter(|e| show_hidden || !e.starts_with('.'))
            .collect();
        if filtered.is_empty() {
            String::new()
        } else {
            format!("{}\n", filtered.join("  "))
        }
    }
}

fn cmd_cd(args: &[&str], current_dir: &mut PathBuf, filesystem: &FakeFilesystem) -> String {
    if args.is_empty() {
        *current_dir = PathBuf::from("/root");
        String::new()
    } else {
        let target = args[0];
        let new_path = if target.starts_with('/') {
            PathBuf::from(target)
        } else if target == ".." {
            current_dir.parent().unwrap_or(current_dir).to_path_buf()
        } else if target == "." {
            current_dir.clone()
        } else {
            current_dir.join(target)
        };

        if filesystem.dir_exists(&new_path) {
            *current_dir = new_path;
            String::new()
        } else {
            format!("cd: {}: No such file or directory\n", target)
        }
    }
}

fn cmd_cat(args: &[&str], filesystem: &FakeFilesystem, current_dir: &PathBuf) -> String {
    if args.is_empty() {
        return "cat: missing operand\n".to_string();
    }

    let path = if args[0].starts_with('/') {
        PathBuf::from(args[0])
    } else {
        current_dir.join(args[0])
    };

    match filesystem.read_file(&path) {
        Some(content) => format!("{}\n", content),
        None => format!("cat: {}: No such file or directory\n", args[0]),
    }
}

fn cmd_echo(args: &[&str], env_vars: &HashMap<String, String>) -> String {
    let mut output = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }
        // Simple variable expansion
        if arg.starts_with('$') {
            let var_name = &arg[1..];
            if let Some(value) = env_vars.get(var_name) {
                output.push_str(value);
            }
        } else {
            output.push_str(arg);
        }
    }
    output.push('\n');
    output
}

fn cmd_env(env_vars: &HashMap<String, String>) -> String {
    let mut output = String::new();
    for (key, value) in env_vars {
        output.push_str(&format!("{}={}\n", key, value));
    }
    output
}

fn cmd_ps() -> String {
    r#"  PID TTY          TIME CMD
    1 pts/0    00:00:00 bash
  234 pts/0    00:00:00 ps
"#
    .to_string()
}

fn cmd_ifconfig() -> String {
    r#"eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
        inet 192.168.1.100  netmask 255.255.255.0  broadcast 192.168.1.255
        inet6 fe80::a00:27ff:fe4e:66a1  prefixlen 64  scopeid 0x20<link>
        ether 08:00:27:4e:66:a1  txqueuelen 1000  (Ethernet)
        RX packets 1234  bytes 567890 (567.8 KB)
        RX errors 0  dropped 0  overruns 0  frame 0
        TX packets 890  bytes 123456 (123.4 KB)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0
"#
    .to_string()
}

fn cmd_ip(args: &[&str]) -> String {
    if args.contains(&"addr") || args.contains(&"a") {
        r#"1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
       valid_lft forever preferred_lft forever
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP group default qlen 1000
    link/ether 08:00:27:4e:66:a1 brd ff:ff:ff:ff:ff:ff
    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0
       valid_lft forever preferred_lft forever
"#
        .to_string()
    } else {
        "Usage: ip [ OPTIONS ] OBJECT { COMMAND | help }\n".to_string()
    }
}

fn cmd_netstat() -> String {
    r#"Active Internet connections (servers and established)
Proto Recv-Q Send-Q Local Address           Foreign Address         State
tcp        0      0 0.0.0.0:22              0.0.0.0:*               LISTEN
tcp        0      0 192.168.1.100:22        192.168.1.50:54321      ESTABLISHED
"#
    .to_string()
}

async fn cmd_wget(args: &[&str]) -> String {
    if args.is_empty() {
        return "wget: missing URL\n".to_string();
    }

    let url = args[args.len() - 1]; // Last arg is usually the URL

    // TODO: Actually download the file and store it for analysis
    format!(
        "--2025-11-09 10:30:15--  {}\n\
        Resolving example.com... 93.184.216.34\n\
        Connecting to example.com|93.184.216.34|:80... connected.\n\
        HTTP request sent, awaiting response... 200 OK\n\
        Length: 1234 (1.2K) [text/html]\n\
        Saving to: 'index.html'\n\
        \n\
        index.html          100%[===================>]   1.20K  --.-KB/s    in 0s\n\
        \n\
        2025-11-09 10:30:15 (45.2 MB/s) - 'index.html' saved [1234/1234]\n",
        url
    )
}

async fn cmd_curl(args: &[&str]) -> String {
    if args.is_empty() {
        return "curl: try 'curl --help' for more information\n".to_string();
    }

    let url = args[args.len() - 1];

    // TODO: Actually download and analyze the content
    format!("<!DOCTYPE html>\n<html>\n<head><title>Example</title></head>\n<body>Downloaded from {}</body>\n</html>\n", url)
}

fn cmd_chmod(args: &[&str]) -> String {
    if args.len() < 2 {
        return "chmod: missing operand\n".to_string();
    }
    // Silently succeed (fake filesystem)
    String::new()
}

fn cmd_chown(args: &[&str]) -> String {
    if args.len() < 2 {
        return "chown: missing operand\n".to_string();
    }
    // Silently succeed (fake filesystem)
    String::new()
}

fn cmd_rm(args: &[&str]) -> String {
    if args.is_empty() {
        return "rm: missing operand\n".to_string();
    }
    // Silently succeed (fake filesystem)
    String::new()
}

fn cmd_mkdir(args: &[&str]) -> String {
    if args.is_empty() {
        return "mkdir: missing operand\n".to_string();
    }
    // Silently succeed (fake filesystem)
    String::new()
}

fn cmd_touch(args: &[&str]) -> String {
    if args.is_empty() {
        return "touch: missing file operand\n".to_string();
    }
    // Silently succeed (fake filesystem)
    String::new()
}

fn cmd_cp(args: &[&str]) -> String {
    if args.len() < 2 {
        return "cp: missing destination file operand\n".to_string();
    }
    // Silently succeed (fake filesystem)
    String::new()
}

fn cmd_mv(args: &[&str]) -> String {
    if args.len() < 2 {
        return "mv: missing destination file operand\n".to_string();
    }
    // Silently succeed (fake filesystem)
    String::new()
}

fn cmd_history() -> String {
    r#"    1  uname -a
    2  whoami
    3  ls -la
    4  cat /etc/passwd
    5  history
"#
    .to_string()
}

fn cmd_exit() -> String {
    "logout\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_whoami() {
        let mut env = HashMap::new();
        env.insert("USER".to_string(), "root".to_string());
        assert_eq!(cmd_whoami(&env), "root\n");
    }

    #[test]
    fn test_cmd_id() {
        assert!(cmd_id().contains("uid=0(root)"));
    }

    #[test]
    fn test_cmd_uname() {
        assert!(cmd_uname(&["-a"]).contains("Linux"));
        assert!(cmd_uname(&["-r"]).contains("5.15"));
    }
}
