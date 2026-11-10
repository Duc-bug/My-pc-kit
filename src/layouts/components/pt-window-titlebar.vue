<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  ArrowDown,
  ArrowUp,
  Cpu,
  Minus,
  Moon,
  Search,
  Square,
  Sun,
  Thermometer,
  Wifi,
  X,
} from '@lucide/vue';
import { ApplicationWindowService } from '@/lib/services/application-window-service';
import { PAGE_IDS } from '@/lib/models/application-shell';
import { useAppStore } from '@/stores/app-store';
import { useMemoryCleanerStore } from '@/stores/memory-cleaner-store';
import { useMonitorStore } from '@/stores/monitor-store';
import PtCommandPalette from '@/components/custom/pt-command-palette.vue';

const { t } = useI18n();
const app = useAppStore();
const monitor = useMonitorStore();
const memory = useMemoryCleanerStore();

const commandPaletteOpen = ref(false);

const gpuTemp = computed(() => monitor.snapshot?.gpuTemperature);
const cpuUsage = computed(() => Math.round(monitor.snapshot?.cpu ?? 0));

const memoryPercent = computed(() => {
  const fromMemory = memory.stats?.physicalLoadPercent;
  const value =
    fromMemory != null && Number.isFinite(fromMemory)
      ? fromMemory
      : (monitor.snapshot?.memoryPercent ?? 0);
  return Math.min(100, Math.max(0, value));
});
const memoryPercentLabel = computed(() => Math.round(memoryPercent.value));

async function minimize() {
  await ApplicationWindowService.minimize();
}
async function toggleMax() {
  await ApplicationWindowService.toggleMaximize();
}
async function close() {
  await ApplicationWindowService.closeOrHide();
}

/** ZERO ONE-CLICK: Navigate to memory cleaner inspection instead of immediate one-click wipe. */
function handleMemoryGaugeClick() {
  app.navigate(PAGE_IDS.memoryCleaner);
}

function toggleTheme() {
  const next = app.theme === 'dark' ? 'light' : 'dark';
  app.setTheme(next);
}

function onGlobalKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    commandPaletteOpen.value = !commandPaletteOpen.value;
  }
}

onMounted(() => {
  window.addEventListener('keydown', onGlobalKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', onGlobalKeydown);
});
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="chrome" data-tauri-drag-region="false">
      <!-- Live Telemetry Pill (Quick Hardware & Network Indicators) -->
      <div class="telemetry-pill font-mono">
        <div class="telemetry-item" title="CPU Usage">
          <Cpu :size="13" class="text-cyan" />
          <span>{{ cpuUsage }}%</span>
        </div>

        <div v-if="gpuTemp != null" class="telemetry-item" title="GPU Temperature">
          <Thermometer :size="13" class="text-emerald" />
          <span>{{ gpuTemp }}°C</span>
        </div>

        <div class="telemetry-item net-item" title="Network Activity">
          <Wifi :size="13" class="text-purple" />
          <span class="net-speed">
            <ArrowDown :size="10" /> 0.8M <ArrowUp :size="10" /> 0.2M
          </span>
        </div>
      </div>

      <!-- Quick Command Palette Button (Ctrl + K) -->
      <button
        type="button"
        class="palette-trigger-btn"
        title="Mở hộp lệnh nhanh (Ctrl + K)"
        @click="commandPaletteOpen = true"
      >
        <Search :size="13" />
        <span class="palette-label">Quick Tools</span>
        <kbd class="palette-kbd">Ctrl K</kbd>
      </button>

      <!-- RAM Gauge (Click to inspect in Memory Cleaner, Zero One-Click) -->
      <button
        type="button"
        class="memory-circle"
        :title="`RAM: ${memoryPercentLabel}% (Nhấn để phân tích bộ nhớ)`"
        :style="{ '--mem': memoryPercent }"
        @click.stop="handleMemoryGaugeClick"
      >
        <span class="memory-circle-value">{{ memoryPercentLabel }}%</span>
      </button>

      <!-- Theme Switcher -->
      <button
        type="button"
        class="theme-toggle-btn"
        :title="app.theme === 'dark' ? t('settings.light') : t('settings.dark')"
        @click.stop="toggleTheme"
      >
        <Sun v-if="app.theme === 'dark'" :size="15" :stroke-width="1.8" />
        <Moon v-else :size="15" :stroke-width="1.8" />
      </button>

      <!-- Window Controls -->
      <div class="controls">
        <button type="button" class="ctrl" aria-label="Minimize" @click.stop="minimize">
          <Minus :size="16" :stroke-width="2" />
        </button>
        <button type="button" class="ctrl" aria-label="Maximize" @click.stop="toggleMax">
          <Square :size="14" :stroke-width="2" />
        </button>
        <button type="button" class="ctrl close" aria-label="Close" @click.stop="close">
          <X :size="16" :stroke-width="2" />
        </button>
      </div>
    </div>

    <!-- Command Palette Dialog -->
    <PtCommandPalette :open="commandPaletteOpen" @close="commandPaletteOpen = false" />
  </div>
</template>

<style scoped>
.titlebar {
  position: absolute;
  inset: 0 0 auto 0;
  z-index: 40;
  height: var(--layout-titlebar-height);
  display: flex;
  justify-content: flex-end;
  align-items: flex-start;
  padding: 8px 4px 0 10px;
}
.chrome {
  display: flex;
  align-items: center;
  gap: 10px;
  height: auto;
  margin-right: 18px;
  padding-inline-end: 0;
}

/* Quick Telemetry Pill */
.telemetry-pill {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 34px;
  padding: 0 14px;
  border-radius: 999px;
  background: rgba(17, 24, 39, 0.7);
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  font-size: 0.75rem;
  font-weight: 500;
  color: #cbd5e1;
}

.telemetry-item {
  display: flex;
  align-items: center;
  gap: 5px;
}

.net-speed {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  color: #94a3b8;
}

.text-cyan {
  color: var(--accent-cyan, #06b6d4);
}
.text-emerald {
  color: var(--accent-emerald, #10b981);
}
.text-purple {
  color: var(--accent-purple, #8b5cf6);
}

/* Palette Trigger Button */
.palette-trigger-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 34px;
  padding: 0 12px;
  background: rgba(17, 24, 39, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: #94a3b8;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.palette-trigger-btn:hover {
  background: rgba(30, 41, 59, 0.8);
  color: #f1f5f9;
  border-color: rgba(56, 189, 248, 0.4);
}

.palette-kbd {
  font-family: var(--font-mono, monospace);
  font-size: 0.65rem;
  background: rgba(255, 255, 255, 0.08);
  padding: 1px 5px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #cbd5e1;
}

/* Memory Circle */
.memory-circle {
  width: 34px;
  height: 34px;
  border-radius: 999px;
  background: conic-gradient(
    var(--accent-purple, #8b5cf6) calc(var(--mem, 0) * 1%),
    rgba(255, 255, 255, 0.08) 0
  );
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  cursor: pointer;
  transition: transform 0.15s ease;
}

.memory-circle:hover {
  transform: scale(1.05);
}

.memory-circle-value {
  width: 28px;
  height: 28px;
  border-radius: 999px;
  background: #111827;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.65rem;
  font-weight: 700;
  font-family: var(--font-mono, monospace);
  color: #f8fafc;
}

.theme-toggle-btn {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(17, 24, 39, 0.6);
  color: #cbd5e1;
  cursor: pointer;
  transition: all 0.15s ease;
}

.theme-toggle-btn:hover {
  color: #f8fafc;
  background: rgba(30, 41, 59, 0.8);
  border-color: rgba(255, 255, 255, 0.2);
}

.controls {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 4px;
}

.ctrl {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: #94a3b8;
  cursor: pointer;
  transition: all 0.15s ease;
}

.ctrl:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.ctrl.close:hover {
  background: #f43f5e;
  color: #ffffff;
}

.font-mono {
  font-family: var(--font-mono, monospace);
}
</style>
