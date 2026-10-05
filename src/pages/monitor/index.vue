<script setup lang="ts">
import { computed, onMounted, onUnmounted, type Component } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  Activity,
  AppWindow,
  CircuitBoard,
  Cpu,
  FileText,
  FolderOpen,
  HardDrive,
  Info,
  LayoutDashboard,
  Network,
  NotebookPen,
  RefreshCw,
  Scissors,
  Settings,
  ShieldCheck,
  Sparkles,
  Terminal,
  Volume2,
  Wrench,
  Zap,
} from '@lucide/vue';
import PtPageShell from '@/components/custom/pt-page-shell.vue';
import { QUICK_ACTIONS, type QuickActionId } from '@/lib/models/actions';
import { formatBytes, formatUptime } from '@/lib/utils/format';
import { useMemoryCleanerStore } from '@/stores/memory-cleaner-store';
import { useMonitorStore } from '@/stores/monitor-store';

const { t } = useI18n();
const store = useMonitorStore();
const memory = useMemoryCleanerStore();
const snap = computed(() => store.snapshot);

const ACTION_ICONS: Record<QuickActionId, Component> = {
  taskManager: AppWindow,
  deviceManager: CircuitBoard,
  controlPanel: LayoutDashboard,
  diskManagement: HardDrive,
  commandPromptAdmin: Terminal,
  powerShellAdmin: Terminal,
  systemInfo: Info,
  registryEditor: NotebookPen,
  settings: Settings,
  services: Wrench,
  fileExplorer: FolderOpen,
  networkConnections: Network,
  snippingTool: Scissors,
  notepad: FileText,
  volumeMixer: Volume2,
};

const memPercent = computed(() => Math.round(snap.value?.memoryPercent ?? 0));
const isHealthy = computed(() => memPercent.value < 80);

async function handleQuickOptimize() {
  if (memory.loading) return;
  await memory.requestOptimize('tray');
  await store.refresh();
}

onMounted(() => store.startPolling());
onUnmounted(() => store.stopPolling());
</script>

<template>
  <PtPageShell :title="t('monitor.title')" :subtitle="t('monitor.subtitle')">
    <template #actions>
      <button type="button" class="pt-btn" :disabled="store.loading" @click="store.refresh()">
        <RefreshCw :size="15" :class="{ spin: store.loading }" />
        {{ t('common.refresh') }}
      </button>
    </template>

    <!-- System Health Banner -->
    <div class="health-banner" :class="{ 'health-banner--heavy': !isHealthy }">
      <div class="health-info">
        <div class="health-icon-box">
          <ShieldCheck v-if="isHealthy" :size="24" class="text-emerald-500" />
          <Zap v-else :size="24" class="text-amber-500" />
        </div>
        <div class="health-texts">
          <h3>
            {{
              isHealthy
                ? t('monitor.systemHealthy')
                : t('monitor.systemHeavy', { percent: memPercent })
            }}
          </h3>
          <p>
            {{ isHealthy ? t('monitor.systemHealthyDesc') : t('monitor.systemHeavyDesc') }}
          </p>
        </div>
      </div>
      <button
        type="button"
        class="optimize-quick-btn"
        :disabled="memory.loading"
        @click="handleQuickOptimize"
      >
        <Sparkles v-if="!memory.loading" :size="16" />
        <RefreshCw v-else :size="16" class="spin" />
        <span>{{ memory.loading ? t('monitor.quickOptimizing') : t('monitor.quickOptimize') }}</span>
      </button>
    </div>

    <!-- Metric Cards -->
    <div class="metrics">
      <div class="card card--cpu">
        <div class="card-top">
          <span class="icon-wrap icon-wrap--blue">
            <Cpu :size="18" />
          </span>
          <span class="card-title">{{ t('monitor.cpu') }}</span>
          <span class="percent-badge">{{ (snap?.cpu ?? 0).toFixed(1) }}%</span>
        </div>
        <div class="value">{{ (snap?.cpu ?? 0).toFixed(1) }}%</div>
        <div class="bar">
          <span class="bar-fill--blue" :style="{ width: `${snap?.cpu ?? 0}%` }" />
        </div>
      </div>

      <div class="card card--memory">
        <div class="card-top">
          <span class="icon-wrap icon-wrap--purple">
            <Activity :size="18" />
          </span>
          <span class="card-title">{{ t('monitor.memory') }}</span>
          <span class="percent-badge">{{ (snap?.memoryPercent ?? 0).toFixed(0) }}%</span>
        </div>
        <div class="value">
          {{ formatBytes(snap?.memoryUsed ?? 0) }} / {{ formatBytes(snap?.memoryTotal ?? 0) }}
        </div>
        <div class="bar">
          <span class="bar-fill--purple" :style="{ width: `${snap?.memoryPercent ?? 0}%` }" />
        </div>
      </div>

      <div class="card card--disk">
        <div class="card-top">
          <span class="icon-wrap icon-wrap--amber">
            <HardDrive :size="18" />
          </span>
          <span class="card-title">{{ t('monitor.disk') }}</span>
          <span class="percent-badge">{{ (snap?.diskPercent ?? 0).toFixed(0) }}%</span>
        </div>
        <div class="value">
          {{ formatBytes(snap?.diskUsed ?? 0) }} / {{ formatBytes(snap?.diskTotal ?? 0) }}
        </div>
        <div class="bar">
          <span class="bar-fill--amber" :style="{ width: `${snap?.diskPercent ?? 0}%` }" />
        </div>
      </div>

      <div class="card card--gpu">
        <div class="card-top">
          <span class="icon-wrap icon-wrap--emerald">
            <CircuitBoard :size="18" />
          </span>
          <span class="card-title">{{ t('monitor.gpu') }}</span>
          <span v-if="snap?.gpuAvailable" class="percent-badge">
            {{ (snap?.gpuUtilization ?? 0).toFixed(0) }}%
          </span>
        </div>
        <div class="value">
          <template v-if="snap?.gpuAvailable">{{ (snap?.gpuUtilization ?? 0).toFixed(0) }}%</template>
          <template v-else>{{ t('monitor.notAvailable') }}</template>
        </div>
        <div v-if="snap?.gpuAvailable" class="bar">
          <span class="bar-fill--emerald" :style="{ width: `${snap?.gpuUtilization ?? 0}%` }" />
        </div>
      </div>
    </div>

    <!-- System Metadata -->
    <div class="meta-card">
      <div class="meta-item">
        <span class="meta-label">{{ t('monitor.uptime') }}</span>
        <strong class="meta-val">{{ formatUptime(snap?.uptimeSeconds ?? 0) }}</strong>
      </div>
      <div class="meta-divider" />
      <div class="meta-item">
        <span class="meta-label">{{ t('monitor.os') }}</span>
        <strong class="meta-val">{{ snap?.osLabel ?? '—' }}</strong>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="section-header">
      <h2 class="section-title">{{ t('monitor.quickActions') }}</h2>
    </div>

    <div class="actions">
      <button
        v-for="action in QUICK_ACTIONS"
        :key="action.id"
        type="button"
        class="action"
        @click="store.openAction(action.id)"
      >
        <span class="action-icon-wrap">
          <component
            :is="ACTION_ICONS[action.id]"
            class="action-icon"
            :size="16"
            :stroke-width="2"
            aria-hidden="true"
          />
        </span>
        <span class="action-name">{{ t(action.labelKey) }}</span>
      </button>
    </div>
  </PtPageShell>
</template>

<style scoped>
/* Health Banner */
.health-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  background: linear-gradient(135deg, #ffffff 0%, #f0fdf4 100%);
  box-shadow: 0 4px 15px -3px rgba(16, 185, 129, 0.08), 0 2px 6px -2px rgba(0, 0, 0, 0.03);
  transition: all 0.2s ease;
}
.health-banner--heavy {
  background: linear-gradient(135deg, #ffffff 0%, #fffbeb 100%);
  border-color: #fde68a;
  box-shadow: 0 4px 15px -3px rgba(245, 158, 11, 0.1), 0 2px 6px -2px rgba(0, 0, 0, 0.03);
}
.health-info {
  display: flex;
  align-items: center;
  gap: 14px;
}
.health-icon-box {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: #ffffff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  flex: none;
}
.health-texts h3 {
  margin: 0;
  font-size: 0.9375rem;
  font-weight: 700;
  color: var(--foreground);
}
.health-texts p {
  margin: 3px 0 0;
  font-size: 0.8125rem;
  color: var(--muted-foreground);
}
.optimize-quick-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  border-radius: 12px;
  border: none;
  background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
  color: #ffffff;
  font-size: 0.8125rem;
  font-weight: 650;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
  transition: all 0.15s ease;
  white-space: nowrap;
}
.optimize-quick-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(37, 99, 235, 0.35);
  background: linear-gradient(135deg, #1d4ed8 0%, #1e40af 100%);
}
.optimize-quick-btn:active:not(:disabled) {
  transform: translateY(0);
}
.optimize-quick-btn:disabled {
  opacity: 0.6;
  cursor: default;
}

/* Metrics Grid */
.metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}
.card {
  position: relative;
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--card);
  box-shadow: var(--shadow-card);
  padding: 18px;
  transition: transform 0.18s ease, box-shadow 0.18s ease;
}
.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px -4px rgba(15, 23, 42, 0.08);
}
.card-top {
  display: flex;
  align-items: center;
  gap: 10px;
}
.card-title {
  color: var(--muted-foreground);
  font-size: 0.8125rem;
  font-weight: 650;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.percent-badge {
  margin-left: auto;
  font-size: 0.75rem;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 999px;
  background: var(--secondary);
  color: var(--secondary-foreground);
}

.icon-wrap {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  flex: none;
}
.icon-wrap--blue {
  background: #eff6ff;
  color: #2563eb;
}
.icon-wrap--purple {
  background: #faf5ff;
  color: #9333ea;
}
.icon-wrap--amber {
  background: #fffbeb;
  color: #d97706;
}
.icon-wrap--emerald {
  background: #f0fdf4;
  color: #059669;
}

.value {
  margin-top: 14px;
  font-size: 1.15rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--foreground);
}

.bar {
  margin-top: 12px;
  height: 8px;
  border-radius: 999px;
  background: var(--secondary);
  overflow: hidden;
}
.bar span {
  display: block;
  height: 100%;
  border-radius: inherit;
  transition: width 0.3s ease;
}
.bar-fill--blue {
  background: linear-gradient(90deg, #3b82f6 0%, #06b6d4 100%);
}
.bar-fill--purple {
  background: linear-gradient(90deg, #a855f7 0%, #ec4899 100%);
}
.bar-fill--amber {
  background: linear-gradient(90deg, #f59e0b 0%, #f97316 100%);
}
.bar-fill--emerald {
  background: linear-gradient(90deg, #10b981 0%, #14b8a6 100%);
}

/* Metadata Card */
.meta-card {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 12px 18px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--card);
  box-shadow: var(--shadow-card);
}
.meta-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8125rem;
}
.meta-label {
  color: var(--muted-foreground);
}
.meta-val {
  color: var(--foreground);
  font-weight: 650;
}
.meta-divider {
  width: 1px;
  height: 16px;
  background: var(--border);
}

/* Quick Actions */
.section-header {
  margin-top: 8px;
}
.section-title {
  margin: 0;
  font-size: 0.9375rem;
  font-weight: 700;
  color: var(--foreground);
  letter-spacing: -0.01em;
}
.actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}
.action {
  display: flex;
  align-items: center;
  gap: 12px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--card);
  color: var(--foreground);
  padding: 12px 14px;
  font-size: 0.8125rem;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
  box-shadow: var(--shadow-card);
  transition: all 0.15s ease;
}
.action-icon-wrap {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: #f1f5f9;
  color: #2563eb;
  flex: none;
  transition: background 0.15s ease, color 0.15s ease;
}
.action:hover {
  transform: translateY(-1px);
  border-color: #2563eb;
  background: #ffffff;
  box-shadow: 0 4px 14px rgba(37, 99, 235, 0.1);
}
.action:hover .action-icon-wrap {
  background: #eff6ff;
  color: #1d4ed8;
}
.action-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.spin {
  animation: spin 0.9s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 900px) {
  .health-banner {
    flex-direction: column;
    align-items: flex-start;
  }
  .optimize-quick-btn {
    width: 100%;
    justify-content: center;
  }
  .metrics,
  .actions {
    grid-template-columns: 1fr;
  }
}
</style>
