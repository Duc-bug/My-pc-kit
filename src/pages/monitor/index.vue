<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { RefreshCw, LayoutGrid, Cpu, HardDrive, Wifi, Clock, Wrench } from '@lucide/vue';
import PtPageShell from '@/components/custom/pt-page-shell.vue';
import { useMonitorStore } from '@/stores/monitor-store';
import PtHardwareBento from './components/pt-hardware-bento.vue';
import PtMultiDriveBento from './components/pt-multi-drive-bento.vue';
import PtSessionUptimeBento from './components/pt-session-uptime-bento.vue';
import PtNetworkDnsBento from './components/pt-network-dns-bento.vue';
import PtSystemDock from './components/pt-system-dock.vue';

const { t } = useI18n();
const monitor = useMonitorStore();

onMounted(() => {
  monitor.startPolling();
});

onUnmounted(() => {
  monitor.stopPolling();
});
</script>

<template>
  <PtPageShell
    :title="t('monitor.title')"
    subtitle="Hệ thống điều khiển & Giám sát tài nguyên thời gian thực (Industrial Bento Grid)"
  >
    <template #actions>
      <button type="button" class="pt-btn" :disabled="monitor.loading" @click="monitor.refresh()">
        <RefreshCw :size="14" :class="{ spin: monitor.loading }" />
        {{ t('common.refresh') }}
      </button>
    </template>

    <div class="bento-dashboard-layout">
      <!-- MODULE A: Real-time Hardware Telemetry (CPU, RAM, GPU Cards) -->
      <section class="bento-section">
        <PtHardwareBento />
      </section>

      <!-- MIDDLE ROW: Multi-Drive Manager (Left) + Session Timeline & Network Telemetry (Right) -->
      <div class="bento-two-columns">
        <!-- MODULE B: Quản lý & Dọn dẹp Đa ổ đĩa (Multi-Drive Manager & Staged Cleaner) -->
        <div class="column-left">
          <PtMultiDriveBento />
        </div>

        <!-- RIGHT STACK: Module C (Session & Uptime) + Module D (Network & Staged DNS) -->
        <div class="column-right">
          <PtSessionUptimeBento />
          <PtNetworkDnsBento />
        </div>
      </div>

      <!-- MODULE E: Compact System Dock -->
      <section class="bento-section">
        <PtSystemDock />
      </section>
    </div>
  </PtPageShell>
</template>

<style scoped>
.bento-dashboard-layout {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-bottom: 50px; /* Offset for floating execution drawer */
}

.bento-section {
  width: 100%;
}

.bento-two-columns {
  display: grid;
  grid-template-columns: 1.15fr 0.85fr;
  gap: 16px;
}

.column-left {
  display: flex;
  flex-direction: column;
}

.column-right {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@media (max-width: 1200px) {
  .bento-two-columns {
    grid-template-columns: 1fr;
  }
}
</style>
