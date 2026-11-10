use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAdapterInfo {
    pub name: String,
    pub description: String,
    pub status: String,
    pub mac_address: String,
    pub ipv4_address: String,
    pub gateway: String,
    pub is_dhcp: bool,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyTargetResult {
    pub name: String,
    pub target: String,
    pub latency_ms: Option<u32>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkTelemetrySnapshot {
    pub adapters: Vec<NetworkAdapterInfo>,
    pub latency: Vec<LatencyTargetResult>,
    pub upload_bytes_per_sec: u64,
    pub download_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsApplyResult {
    pub success: bool,
    pub command_executed: String,
    pub output: String,
    pub error: Option<String>,
}

/// Query active physical network adapters and their current IPv4 / DNS config.
pub fn get_network_adapters() -> Vec<NetworkAdapterInfo> {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$adapters = @()

$configs = Get-NetIPConfiguration | Where-Object {
    $_.IPv4DefaultGateway -ne $null -and
    $_.InterfaceAlias -notmatch 'Virtual|vEthernet|WSL|Docker|VPN|Tap|Loopback|Npcap'
}

foreach ($c in $configs) {
    $alias = $c.InterfaceAlias
    $adapter = Get-NetAdapter -Name $alias -ErrorAction SilentlyContinue
    if (-not $adapter -or $adapter.Virtual) { continue }

    $ip = if ($c.IPv4Address) { $c.IPv4Address.IPAddress } else { '0.0.0.0' }
    $gw = if ($c.IPv4DefaultGateway) { $c.IPv4DefaultGateway.NextHop } else { '' }
    $dns = @()
    if ($c.DNSServer) {
        foreach ($d in $c.DNSServer.ServerAddresses) {
            if ($d -match '^\d+\.\d+\.\d+\.\d+$') { $dns += $d }
        }
    }

    $dhcp = $true
    try {
        $dhcp = (Get-NetIPInterface -InterfaceAlias $alias -AddressFamily IPv4 -ErrorAction SilentlyContinue).Dhcp -eq 'Enabled'
    } catch {}

    $adapters += [pscustomobject]@{
        name = $alias
        description = if ($adapter.InterfaceDescription) { $adapter.InterfaceDescription.Trim() } else { $alias }
        status = if ($adapter.Status) { "$($adapter.Status)" } else { 'Up' }
        macAddress = if ($adapter.MacAddress) { $adapter.MacAddress } else { '' }
        ipv4Address = $ip
        gateway = $gw
        isDhcp = $dhcp
        dnsServers = @($dns)
    }
}

if ($adapters.Count -eq 0) {
    # Fallback to standard Get-NetAdapter
    Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and -not $_.Virtual } | ForEach-Object {
        $adapters += [pscustomobject]@{
            name = $_.Name
            description = $_.InterfaceDescription
            status = "$($_.Status)"
            macAddress = $_.MacAddress
            ipv4Address = 'Connected'
            gateway = ''
            isDhcp = $true
            dnsServers = @('1.1.1.1', '8.8.8.8')
        }
    }
}

$adapters | ConvertTo-Json -Compress
"#;

    let mut cmd = crate::process::powershell_cmd();
    cmd.args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script]);
    crate::process::hide_console(&mut cmd);
    let output = match cmd.output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Vec::new();
    }

    // Handle single object vs array in Json
    if text.starts_with('{') {
        if let Ok(single) = serde_json::from_str::<NetworkAdapterInfo>(&text) {
            return vec![single];
        }
    }
    serde_json::from_str::<Vec<NetworkAdapterInfo>>(&text).unwrap_or_default()
}

/// Measure multi-node ping latency (Gateway, Cloudflare 1.1.1.1, Google 8.8.8.8).
pub fn test_network_latency(gateway_ip: Option<&str>) -> Vec<LatencyTargetResult> {
    let gw = gateway_ip.unwrap_or("192.168.1.1");
    let script = format!(
        r#"
$targets = @(
    @{{ name = "Local Gateway"; target = "{gw}" }},
    @{{ name = "Cloudflare DNS"; target = "1.1.1.1" }},
    @{{ name = "Google DNS"; target = "8.8.8.8" }}
)

$results = @()
foreach ($t in $targets) {{
    $ping = Test-Connection -ComputerName $t.target -Count 1 -TimeoutSeconds 2 -ErrorAction SilentlyContinue
    $ms = if ($ping -and $null -ne $ping.ResponseTime) {{ [uint32]$ping.ResponseTime }} else {{ $null }}
    $results += [pscustomobject]@{{
        name = $t.name
        target = $t.target
        latencyMs = $ms
        status = if ($null -ne $ms) {{ "Online" }} else {{ "Unreachable" }}
    }}
}}
$results | ConvertTo-Json -Compress
"#
    );

    let mut cmd = crate::process::powershell_cmd();
    cmd.args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script]);
    crate::process::hide_console(&mut cmd);
    let output = match cmd.output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Vec::new();
    }

    serde_json::from_str::<Vec<LatencyTargetResult>>(&text).unwrap_or_default()
}

/// Staged DNS configuration application:
/// Executes `netsh` or `Set-DnsClientServerAddress` and optionally `ipconfig /flushdns`.
/// Returns the exact commands executed and stdout/stderr for the Execution Action Drawer.
pub fn apply_dns_configuration(
    interface_name: &str,
    primary_dns: Option<&str>,
    secondary_dns: Option<&str>,
    use_dhcp: bool,
    flush_dns: bool,
) -> DnsApplyResult {
    let mut executed_commands = Vec::new();
    let mut combined_output = String::new();

    if use_dhcp {
        // Set DNS to DHCP
        let cmd_str = format!(
            "netsh interface ip set dns name=\"{}\" source=dhcp",
            interface_name
        );
        executed_commands.push(cmd_str.clone());

        let mut cmd = Command::new("netsh");
        cmd.args(["interface", "ip", "set", "dns", &format!("name=\"{}\"", interface_name), "source=dhcp"]);
        crate::process::hide_console(&mut cmd);
        match cmd.output() {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                combined_output.push_str(&format!("$ {cmd_str}\n{stdout}{stderr}\n"));
            }
            Err(e) => {
                return DnsApplyResult {
                    success: false,
                    command_executed: cmd_str,
                    output: combined_output,
                    error: Some(format!("Failed to execute netsh: {e}")),
                };
            }
        }
    } else if let Some(primary) = primary_dns {
        // Primary DNS
        let cmd1_str = format!(
            "netsh interface ip set dns name=\"{}\" static {} primary",
            interface_name, primary
        );
        executed_commands.push(cmd1_str.clone());

        let mut cmd1 = Command::new("netsh");
        cmd1.args([
            "interface",
            "ip",
            "set",
            "dns",
            &format!("name=\"{}\"", interface_name),
            "static",
            primary,
            "primary",
        ]);
        crate::process::hide_console(&mut cmd1);
        if let Ok(o) = cmd1.output() {
            let out = String::from_utf8_lossy(&o.stdout);
            let err = String::from_utf8_lossy(&o.stderr);
            combined_output.push_str(&format!("$ {cmd1_str}\n{out}{err}\n"));
        }

        // Secondary DNS
        if let Some(sec) = secondary_dns {
            if !sec.is_empty() {
                let cmd2_str = format!(
                    "netsh interface ip add dns name=\"{}\" {} index=2",
                    interface_name, sec
                );
                executed_commands.push(cmd2_str.clone());

                let mut cmd2 = Command::new("netsh");
                cmd2.args([
                    "interface",
                    "ip",
                    "add",
                    "dns",
                    &format!("name=\"{}\"", interface_name),
                    sec,
                    "index=2",
                ]);
                crate::process::hide_console(&mut cmd2);
                if let Ok(o) = cmd2.output() {
                    let out = String::from_utf8_lossy(&o.stdout);
                    let err = String::from_utf8_lossy(&o.stderr);
                    combined_output.push_str(&format!("$ {cmd2_str}\n{out}{err}\n"));
                }
            }
        }
    }

    // Flush DNS Cache if requested
    if flush_dns {
        let flush_cmd_str = "ipconfig /flushdns".to_string();
        executed_commands.push(flush_cmd_str.clone());

        let mut flush_cmd = Command::new("ipconfig");
        flush_cmd.arg("/flushdns");
        crate::process::hide_console(&mut flush_cmd);
        if let Ok(o) = flush_cmd.output() {
            let out = String::from_utf8_lossy(&o.stdout);
            let err = String::from_utf8_lossy(&o.stderr);
            combined_output.push_str(&format!("$ {flush_cmd_str}\n{out}{err}\n"));
        }
    }

    DnsApplyResult {
        success: true,
        command_executed: executed_commands.join(" && "),
        output: combined_output,
        error: None,
    }
}

