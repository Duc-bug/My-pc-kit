use pctoolkit_platform::{
    apply_dns_configuration, get_network_adapters, test_network_latency, DnsApplyResult,
    LatencyTargetResult, NetworkAdapterInfo,
};

#[tauri::command]
pub fn get_network_info() -> Vec<NetworkAdapterInfo> {
    get_network_adapters()
}

#[tauri::command]
pub fn get_network_latency(gateway_ip: Option<String>) -> Vec<LatencyTargetResult> {
    test_network_latency(gateway_ip.as_deref())
}

#[tauri::command]
pub fn set_dns_configuration(
    interface_name: String,
    primary_dns: Option<String>,
    secondary_dns: Option<String>,
    use_dhcp: bool,
    flush_dns: bool,
) -> DnsApplyResult {
    apply_dns_configuration(
        &interface_name,
        primary_dns.as_deref(),
        secondary_dns.as_deref(),
        use_dhcp,
        flush_dns,
    )
}

