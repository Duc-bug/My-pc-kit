<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import {
  Wifi,
  Globe,
  Radio,
  RefreshCw,
  Check,
  X,
  Server,
  Zap,
  RotateCw,
  ShieldCheck,
  Activity,
  AlertCircle,
} from '@lucide/vue';
import {
  NetworkService,
  type NetworkAdapterInfo,
  type LatencyTargetResult,
} from '@/lib/services/api-services';
import { useExecutionLogStore } from '@/stores/execution-log-store';

const logStore = useExecutionLogStore();

const adapters = ref<NetworkAdapterInfo[]>([]);
const selectedAdapterName = ref<string>('');
const latencyResults = ref<LatencyTargetResult[]>([
  { name: 'Gateway', target: '192.168.1.1', latencyMs: 2, status: 'Online' },
  { name: 'Cloudflare', target: '1.1.1.1', latencyMs: 24, status: 'Online' },
  { name: 'Google', target: '8.8.8.8', latencyMs: 28, status: 'Online' },
]);

const loadingAdapters = ref(false);
const pinging = ref(false);
const applyingDns = ref(false);

// Preset DNS configs
const DNS_PRESETS = [
  { id: 'dhcp', label: 'Tự động (DHCP)', primary: '', secondary: '' },
  { id: 'cloudflare', label: 'Cloudflare (1.1.1.1 / 1.0.0.1)', primary: '1.1.1.1', secondary: '1.0.0.1' },
  { id: 'google', label: 'Google (8.8.8.8 / 8.8.4.4)', primary: '8.8.8.8', secondary: '8.8.4.4' },
  { id: 'quad9', label: 'Quad9 (9.9.9.9 / 149.112.112.112)', primary: '9.9.9.9', secondary: '149.112.112.112' },
  { id: 'custom', label: 'Tùy chỉnh (Custom IP)', primary: '', secondary: '' },
];

const selectedPreset = ref('dhcp');
const customPrimaryDns = ref('');
const customSecondaryDns = ref('');
const autoFlushDns = ref(true);

let pingTimer: number | null = null;

const currentAdapter = computed(() => {
  return adapters.value.find((a) => a.name === selectedAdapterName.value) ?? adapters.value[0];
});

// Dirty State Detection
const isDirty = computed(() => {
  if (!currentAdapter.value) return false;
  if (selectedPreset.value === 'dhcp') {
    return !currentAdapter.value.isDhcp;
  }
  const targetPrimary = selectedPreset.value === 'custom'
    ? customPrimaryDns.value.trim()
    : DNS_PRESETS.find((p) => p.id === selectedPreset.value)?.primary || '';
  
  const currentPrimary = currentAdapter.value.dnsServers[0] || '';
  return targetPrimary !== '' && targetPrimary !== currentPrimary;
});

const dirtySummaryText = computed(() => {
  const adapterName = currentAdapter.value?.name || 'Network';
  const target = selectedPreset.value === 'dhcp'
    ? 'DHCP'
    : selectedPreset.value === 'custom'
      ? customPrimaryDns.value
      : DNS_PRESETS.find((p) => p.id === selectedPreset.value)?.primary;
  return `Thay đổi DNS của ${adapterName} ➔ ${target}`;
});

async function loadNetworkData() {
  loadingAdapters.value = true;
  try {
    const list = await NetworkService.getAdapters();
    adapters.value = list;
    if (list.length > 0 && !selectedAdapterName.value) {
      selectedAdapterName.value = list[0].name;
      syncFormWithAdapter(list[0]);
    }
  } catch (err) {
    console.error('Failed to load network adapters:', err);
  } finally {
    loadingAdapters.value = false;
  }
}

function syncFormWithAdapter(adapter: NetworkAdapterInfo) {
  if (adapter.isDhcp) {
    selectedPreset.value = 'dhcp';
  } else {
    const primary = adapter.dnsServers[0];
    if (primary === '1.1.1.1') selectedPreset.value = 'cloudflare';
    else if (primary === '8.8.8.8') selectedPreset.value = 'google';
    else if (primary === '9.9.9.9') selectedPreset.value = 'quad9';
    else {
      selectedPreset.value = 'custom';
      customPrimaryDns.value = primary || '';
      customSecondaryDns.value = adapter.dnsServers[1] || '';
    }
  }
}

watch(selectedAdapterName, (name) => {
  const ad = adapters.value.find((a) => a.name === name);
  if (ad) syncFormWithAdapter(ad);
});

async function runPingLatency() {
  if (pinging.value) return;
  pinging.value = true;
  try {
    const gw = currentAdapter.value?.gateway || undefined;
    const res = await NetworkService.getLatency(gw);
    if (res.length > 0) {
      latencyResults.value = res;
    }
  } catch (err) {
    // Continue
  } finally {
    pinging.value = false;
  }
}

function revertChanges() {
  if (currentAdapter.value) {
    syncFormWithAdapter(currentAdapter.value);
  }
}

/** 2-STEP STAGED EXECUTION: Apply DNS Configuration */
async function applyDnsConfiguration() {
  if (!currentAdapter.value || applyingDns.value) return;
  applyingDns.value = true;

  const isDhcp = selectedPreset.value === 'dhcp';
  let primary: string | undefined;
  let secondary: string | undefined;

  if (!isDhcp) {
    if (selectedPreset.value === 'custom') {
      primary = customPrimaryDns.value.trim();
      secondary = customSecondaryDns.value.trim() || undefined;
    } else {
      const p = DNS_PRESETS.find((x) => x.id === selectedPreset.value);
      primary = p?.primary;
      secondary = p?.secondary;
    }
  }

  const logId = logStore.addLog(
    `Cấu hình DNS: ${currentAdapter.value.name}`,
    isDhcp
      ? `netsh interface ip set dns name="${currentAdapter.value.name}" source=dhcp`
      : `netsh interface ip set dns name="${currentAdapter.value.name}" static ${primary} primary`,
    'running'
  );

  try {
    const res = await NetworkService.setDnsConfiguration({
      interfaceName: currentAdapter.value.name,
      primaryDns: primary,
      secondaryDns: secondary,
      useDhcp: isDhcp,
      flushDns: autoFlushDns.value,
    });

    logStore.updateLog(logId, {
      status: res.success ? 'success' : 'error',
      output: res.output || (res.success ? 'Đã áp dụng DNS thành công' : res.error || 'Lỗi không xác định'),
    });

    await loadNetworkData();
  } catch (err: any) {
    logStore.updateLog(logId, {
      status: 'error',
      output: `Lỗi áp dụng DNS: ${err?.message || err}`,
    });
  } finally {
    applyingDns.value = false;
  }
}

onMounted(() => {
  void loadNetworkData();
  void runPingLatency();
  pingTimer = window.setInterval(runPingLatency, 5000);
});

onUnmounted(() => {
  if (pingTimer) window.clearInterval(pingTimer);
});
</script>

<template>
  <div class="network-bento-card">
    <div class="card-header">
      <div class="header-title">
        <Wifi :size="16" class="text-purple" />
        <span>NETWORK TELEMETRY & STAGED DNS MANAGER</span>
      </div>
      <button
        type="button"
        class="refresh-btn"
        :disabled="loadingAdapters"
        title="Làm mới mạng"
        @click="loadNetworkData"
      >
        <RotateCw :size="13" :class="{ spin: loadingAdapters }" />
      </button>
    </div>

    <!-- Multi-Node Latency Ping Indicator (Updated every 5s) -->
    <div class="latency-grid font-mono">
      <div
        v-for="target in latencyResults"
        :key="target.name"
        class="latency-item"
        :class="{
          'latency-item--fast': target.latencyMs != null && target.latencyMs < 35,
          'latency-item--med': target.latencyMs != null && target.latencyMs >= 35 && target.latencyMs < 90,
          'latency-item--slow': target.latencyMs == null || target.latencyMs >= 90
        }"
      >
        <div class="target-info">
          <span class="target-name">{{ target.name }}</span>
          <span class="target-ip">{{ target.target }}</span>
        </div>
        <div class="target-metric">
          <span class="ping-ms">
            {{ target.latencyMs != null ? `${target.latencyMs} ms` : 'Offline' }}
          </span>
          <span class="ping-dot" />
        </div>
      </div>
    </div>

    <!-- Staged DNS Form -->
    <div class="dns-form-container">
      <div class="adapter-select-row">
        <label class="form-label">Giao diện mạng vật lý:</label>
        <select v-model="selectedAdapterName" class="adapter-select font-mono">
          <option v-for="ad in adapters" :key="ad.name" :value="ad.name">
            {{ ad.name }} ({{ ad.ipv4Address }})
          </option>
        </select>
      </div>

      <!-- Current Adapter Stats -->
      <div v-if="currentAdapter" class="current-dns-strip font-mono">
        <div class="strip-item">
          <span class="k">DHCP:</span>
          <span class="v" :class="currentAdapter.isDhcp ? 'text-emerald' : 'text-amber'">
            {{ currentAdapter.isDhcp ? 'Enabled' : 'Static' }}
          </span>
        </div>
        <div class="strip-item">
          <span class="k">DNS hiện tại:</span>
          <span class="v">{{ currentAdapter.dnsServers.join(', ') || 'Auto / Default Gateway' }}</span>
        </div>
      </div>

      <!-- DNS Presets Radio Grid -->
      <div class="presets-grid">
        <label
          v-for="preset in DNS_PRESETS"
          :key="preset.id"
          class="preset-radio-card"
          :class="{ 'preset-radio-card--selected': selectedPreset === preset.id }"
        >
          <input
            v-model="selectedPreset"
            type="radio"
            name="dns-preset"
            :value="preset.id"
            class="radio-input"
          />
          <span class="preset-name">{{ preset.label }}</span>
        </label>
      </div>

      <!-- Custom IP Inputs if Custom Selected -->
      <div v-if="selectedPreset === 'custom'" class="custom-inputs-row font-mono">
        <input
          v-model="customPrimaryDns"
          type="text"
          placeholder="Primary DNS (VD: 1.1.1.1)"
          class="custom-ip-input"
        />
        <input
          v-model="customSecondaryDns"
          type="text"
          placeholder="Secondary DNS (VD: 1.0.0.1)"
          class="custom-ip-input"
        />
      </div>

      <!-- Flush DNS Cache Option -->
      <label class="flush-dns-checkbox">
        <input v-model="autoFlushDns" type="checkbox" />
        <span>Tự động Flush DNS Cache (<code class="font-mono">ipconfig /flushdns</code>) sau khi áp dụng</span>
      </label>
    </div>

    <!-- Dirty State Action Bar (Appears only when changes are made) -->
    <div v-if="isDirty" class="dirty-state-bar font-mono">
      <div class="dirty-summary">
        <AlertCircle :size="15" class="text-amber" />
        <span>{{ dirtySummaryText }}</span>
      </div>
      <div class="dirty-buttons">
        <button type="button" class="pt-btn pt-btn-secondary pt-btn-sm" @click="revertChanges">
          <X :size="13" /> Hủy
        </button>
        <button
          type="button"
          class="pt-btn pt-btn-primary pt-btn-sm"
          :disabled="applyingDns"
          @click="applyDnsConfiguration"
        >
          <Check :size="13" />
          {{ applyingDns ? 'Đang áp dụng…' : 'Áp dụng cấu hình' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.network-bento-card {
  background: var(--card-glass, rgba(17, 24, 39, 0.75));
  backdrop-filter: blur(12px);
  border: 1px solid var(--border, rgba(255, 255, 255, 0.07));
  border-radius: 12px;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  transition: all 0.2s ease;
}

.network-bento-card:hover {
  border-color: var(--border-hover, rgba(59, 130, 246, 0.4));
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  color: #cbd5e1;
}

.refresh-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.refresh-btn:hover {
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.08);
}

/* Latency Grid */
.latency-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}

.latency-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.04);
  padding: 8px 10px;
  border-radius: 6px;
}

.target-info {
  display: flex;
  flex-direction: column;
}

.target-name {
  font-size: 0.725rem;
  font-weight: 600;
  color: #f1f5f9;
}

.target-ip {
  font-size: 0.65rem;
  color: #64748b;
}

.target-metric {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ping-ms {
  font-size: 0.75rem;
  font-weight: 600;
}

.latency-item--fast .ping-ms { color: #10b981; }
.latency-item--med .ping-ms { color: #f59e0b; }
.latency-item--slow .ping-ms { color: #f43f5e; }

.ping-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: currentColor;
}

.latency-item--fast .ping-dot { background: #10b981; }
.latency-item--med .ping-dot { background: #f59e0b; }
.latency-item--slow .ping-dot { background: #f43f5e; }

/* DNS Form */
.dns-form-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: rgba(0, 0, 0, 0.2);
  padding: 12px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.adapter-select-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.form-label {
  font-size: 0.75rem;
  color: #94a3b8;
}

.adapter-select {
  background: #111827;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #f1f5f9;
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 0.75rem;
  outline: none;
}

.current-dns-strip {
  display: flex;
  gap: 18px;
  font-size: 0.7rem;
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 4px;
}

.strip-item .k {
  color: #64748b;
  margin-right: 4px;
}

.presets-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}

.preset-radio-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.725rem;
  color: #cbd5e1;
  transition: all 0.12s ease;
}

.preset-radio-card:hover,
.preset-radio-card--selected {
  background: rgba(56, 189, 248, 0.08);
  border-color: rgba(56, 189, 248, 0.3);
  color: #f8fafc;
}

.preset-radio-card--selected {
  font-weight: 600;
}

.radio-input {
  cursor: pointer;
}

.custom-inputs-row {
  display: flex;
  gap: 8px;
}

.custom-ip-input {
  flex: 1;
  background: #111827;
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #f8fafc;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 0.75rem;
  outline: none;
}

.custom-ip-input:focus {
  border-color: #38bdf8;
}

.flush-dns-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.725rem;
  color: #94a3b8;
  cursor: pointer;
}

.flush-dns-checkbox code {
  color: #cbd5e1;
}

/* Dirty State Bar */
.dirty-state-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-radius: 6px;
  animation: slide-up 0.15s ease;
}

@keyframes slide-up {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.dirty-summary {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  color: #fcd34d;
  font-weight: 500;
}

.dirty-buttons {
  display: flex;
  align-items: center;
  gap: 6px;
}

.pt-btn-sm {
  padding: 4px 10px;
  font-size: 0.725rem;
}

.text-purple { color: var(--accent-purple, #8b5cf6); }
.text-emerald { color: #10b981; }
.text-amber { color: #f59e0b; }

.font-mono {
  font-family: var(--font-mono, monospace);
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
