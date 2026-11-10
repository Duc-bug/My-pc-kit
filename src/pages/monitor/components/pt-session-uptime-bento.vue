<script setup lang="ts">
import { computed } from 'vue';
import { Clock, ShieldAlert, CheckCircle2, AlertTriangle, Zap, Server } from '@lucide/vue';
import { useMonitorStore } from '@/stores/monitor-store';

const monitor = useMonitorStore();

const uptimeSec = computed(() => monitor.snapshot?.uptimeSeconds ?? 0);

const days = computed(() => Math.floor(uptimeSec.value / 86400));
const hours = computed(() => Math.floor((uptimeSec.value % 86400) / 3600));
const minutes = computed(() => Math.floor((uptimeSec.value % 3600) / 60));
const seconds = computed(() => uptimeSec.value % 60);

// Boot time calculation
const bootTimeFormatted = computed(() => {
  const bootTimestamp = Date.now() - uptimeSec.value * 1000;
  const d = new Date(bootTimestamp);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `Khởi động lúc: ${pad(d.getHours())}:${pad(d.getMinutes())} - ${pad(d.getDate())}/${pad(d.getMonth() + 1)}/${d.getFullYear()}`;
});

// Session health state
const healthTier = computed<'healthy' | 'continuous' | 'heavy'>(() => {
  const h = uptimeSec.value / 3600;
  if (h < 8) return 'healthy';
  if (h < 24) return 'continuous';
  return 'heavy';
});

// Mini 7-day uptime bar simulation
const weeklyUptimeDays = [
  { day: 'T2', hours: 7.5 },
  { day: 'T3', hours: 10.2 },
  { day: 'T4', hours: 8.8 },
  { day: 'T5', hours: 6.4 },
  { day: 'T6', hours: 12.0 },
  { day: 'T7', hours: 9.1 },
  { day: 'CN', hours: Math.min(24, Math.round((uptimeSec.value / 3600) * 10) / 10) },
];
</script>

<template>
  <div class="uptime-card">
    <div class="card-header">
      <div class="header-title">
        <Clock :size="16" class="text-cyan" />
        <span>SESSION & UPTIME TIMELINE</span>
      </div>
      <span
        class="health-badge font-mono"
        :class="{
          'health-badge--healthy': healthTier === 'healthy',
          'health-badge--continuous': healthTier === 'continuous',
          'health-badge--heavy': healthTier === 'heavy'
        }"
      >
        <CheckCircle2 v-if="healthTier === 'healthy'" :size="12" />
        <Zap v-else-if="healthTier === 'continuous'" :size="12" />
        <AlertTriangle v-else :size="12" />
        {{ healthTier === 'healthy' ? 'Ổn định' : healthTier === 'continuous' ? 'Hoạt động liên tục' : 'Cần làm mới cache' }}
      </span>
    </div>

    <!-- Monospace Clock -->
    <div class="clock-display font-mono">
      <div class="clock-unit">
        <span class="unit-val">{{ String(days).padStart(2, '0') }}</span>
        <span class="unit-tag">NGÀY</span>
      </div>
      <span class="colon">:</span>
      <div class="clock-unit">
        <span class="unit-val">{{ String(hours).padStart(2, '0') }}</span>
        <span class="unit-tag">GIỜ</span>
      </div>
      <span class="colon">:</span>
      <div class="clock-unit">
        <span class="unit-val">{{ String(minutes).padStart(2, '0') }}</span>
        <span class="unit-tag">PHÚT</span>
      </div>
      <span class="colon">:</span>
      <div class="clock-unit">
        <span class="unit-val text-cyan">{{ String(seconds).padStart(2, '0') }}</span>
        <span class="unit-tag">GIÂY</span>
      </div>
    </div>

    <div class="boot-time-strip font-mono">
      <span>{{ bootTimeFormatted }}</span>
      <span class="bios-time">Last BIOS Time: 11.4s</span>
    </div>

    <!-- Health Indicator Bar -->
    <div class="health-indicator-box">
      <div class="health-bar">
        <div
          class="health-bar-fill"
          :class="{
            'health-bar-fill--healthy': healthTier === 'healthy',
            'health-bar-fill--continuous': healthTier === 'continuous',
            'health-bar-fill--heavy': healthTier === 'heavy'
          }"
          :style="{ width: `${Math.min(100, Math.max(15, (uptimeSec / 86400) * 100))}%` }"
        />
      </div>
      <div class="health-legend font-mono">
        <span :class="{ active: healthTier === 'healthy' }">&lt; 8h Tối ưu</span>
        <span :class="{ active: healthTier === 'continuous' }">8 - 24h Liên tục</span>
        <span :class="{ active: healthTier === 'heavy' }">&gt; 24h Làm mới cache</span>
      </div>
    </div>

    <!-- Weekly mini bar chart -->
    <div class="weekly-section font-mono">
      <span class="weekly-title">Lịch sử hoạt động 7 ngày gần nhất</span>
      <div class="weekly-bars">
        <div v-for="item in weeklyUptimeDays" :key="item.day" class="bar-col">
          <div class="bar-track">
            <div class="bar-fill" :style="{ height: `${Math.min(100, (item.hours / 24) * 100)}%` }" />
          </div>
          <span class="day-label">{{ item.day }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.uptime-card {
  background: var(--card-glass, rgba(17, 24, 39, 0.75));
  backdrop-filter: blur(12px);
  border: 1px solid var(--border, rgba(255, 255, 255, 0.07));
  border-radius: 12px;
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: all 0.2s ease;
}

.uptime-card:hover {
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

.health-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.675rem;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
}

.health-badge--healthy {
  background: rgba(16, 185, 129, 0.12);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.25);
}

.health-badge--continuous {
  background: rgba(245, 158, 11, 0.12);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.25);
}

.health-badge--heavy {
  background: rgba(244, 63, 94, 0.12);
  color: #f43f5e;
  border: 1px solid rgba(244, 63, 94, 0.25);
}

/* Clock Display */
.clock-display {
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 8px;
  padding: 12px 0 8px;
  background: rgba(0, 0, 0, 0.25);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.03);
}

.clock-unit {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.unit-val {
  font-size: 1.85rem;
  font-weight: 700;
  color: #f8fafc;
  line-height: 1;
}

.unit-tag {
  font-size: 0.6rem;
  color: #64748b;
  margin-top: 4px;
  letter-spacing: 0.05em;
}

.colon {
  font-size: 1.4rem;
  font-weight: 700;
  color: #475569;
  line-height: 1;
}

.boot-time-strip {
  display: flex;
  justify-content: space-between;
  font-size: 0.725rem;
  color: #94a3b8;
  padding: 0 4px;
}

.bios-time {
  color: var(--accent-cyan, #06b6d4);
}

/* Health Bar */
.health-indicator-box {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.health-bar {
  height: 5px;
  background: rgba(255, 255, 255, 0.07);
  border-radius: 999px;
  overflow: hidden;
}

.health-bar-fill {
  height: 100%;
  border-radius: 999px;
  transition: width 0.3s ease;
}

.health-bar-fill--healthy { background: #10b981; }
.health-bar-fill--continuous { background: #f59e0b; }
.health-bar-fill--heavy { background: #f43f5e; }

.health-legend {
  display: flex;
  justify-content: space-between;
  font-size: 0.65rem;
  color: #475569;
}

.health-legend .active {
  color: #cbd5e1;
  font-weight: 600;
}

/* Weekly Mini Chart */
.weekly-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

.weekly-title {
  font-size: 0.7rem;
  color: #64748b;
}

.weekly-bars {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  height: 48px;
  padding: 0 4px;
}

.bar-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  height: 100%;
}

.bar-track {
  width: 14px;
  flex: 1;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
  display: flex;
  align-items: flex-end;
  overflow: hidden;
}

.bar-fill {
  width: 100%;
  background: var(--accent-cyan, #06b6d4);
  border-radius: 3px 3px 0 0;
  opacity: 0.75;
}

.day-label {
  font-size: 0.65rem;
  color: #64748b;
}

.text-cyan { color: var(--accent-cyan, #06b6d4); }
.font-mono { font-family: var(--font-mono, monospace); }
</style>
