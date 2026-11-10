<script setup lang="ts">
import { computed } from 'vue';
import { useExecutionLogStore } from '@/stores/execution-log-store';
import { Terminal, ChevronUp, ChevronDown, Trash2, CheckCircle2, AlertCircle, Clock } from '@lucide/vue';

const store = useExecutionLogStore();

const latestLog = computed(() => store.latestLog);
</script>

<template>
  <div class="execution-drawer" :class="{ 'execution-drawer--open': store.drawerOpen }">
    <!-- Collapsed Floating Bar / Header -->
    <div class="drawer-header" @click="store.toggleDrawer()">
      <div class="header-left">
        <Terminal :size="15" class="terminal-icon" />
        <span class="header-title">System Action Log</span>
        <span v-if="store.logs.length" class="log-count">{{ store.logs.length }}</span>
        <span v-if="latestLog" class="latest-summary">
          <span class="muted font-mono">[{{ latestLog.timestamp }}]</span>
          <span class="latest-action font-mono">{{ latestLog.action }}</span>
          <span
            class="status-pill"
            :class="{
              'status-pill--success': latestLog.status === 'success',
              'status-pill--error': latestLog.status === 'error',
              'status-pill--running': latestLog.status === 'running'
            }"
          >
            {{ latestLog.status.toUpperCase() }}
          </span>
        </span>
      </div>
      <div class="header-right">
        <button
          v-if="store.logs.length"
          type="button"
          class="icon-btn"
          title="Clear logs"
          @click.stop="store.clearLogs()"
        >
          <Trash2 :size="14" />
        </button>
        <button type="button" class="icon-btn" :title="store.drawerOpen ? 'Thu gọn' : 'Mở rộng'">
          <ChevronDown v-if="store.drawerOpen" :size="16" />
          <ChevronUp v-else :size="16" />
        </button>
      </div>
    </div>

    <!-- Expanded Terminal View -->
    <div v-if="store.drawerOpen" class="drawer-body">
      <div v-if="store.logs.length === 0" class="empty-logs">
        <Terminal :size="28" class="empty-icon" />
        <p>Chưa có lệnh can thiệp hệ thống nào được thực thi.</p>
        <span class="empty-hint">Mọi tác vụ đổi DNS, tắt tiến trình, dọn dẹp ổ đĩa sẽ hiển thị câu lệnh và stdout tại đây.</span>
      </div>

      <div v-else class="logs-container font-mono">
        <div
          v-for="log in store.logs"
          :key="log.id"
          class="log-card"
          :class="{
            'log-card--success': log.status === 'success',
            'log-card--error': log.status === 'error',
            'log-card--running': log.status === 'running'
          }"
        >
          <div class="log-top">
            <div class="log-meta">
              <span class="timestamp">{{ log.timestamp }}</span>
              <span class="action-tag">{{ log.action }}</span>
            </div>
            <span
              class="status-badge"
              :class="{
                'status-badge--success': log.status === 'success',
                'status-badge--error': log.status === 'error',
                'status-badge--running': log.status === 'running'
              }"
            >
              <CheckCircle2 v-if="log.status === 'success'" :size="12" />
              <AlertCircle v-else-if="log.status === 'error'" :size="12" />
              <Clock v-else :size="12" class="spin" />
              {{ log.status.toUpperCase() }}
            </span>
          </div>

          <div class="log-cmd">
            <span class="prompt">$</span>
            <code>{{ log.command }}</code>
          </div>

          <div v-if="log.output" class="log-output">
            <pre>{{ log.output }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.execution-drawer {
  position: fixed;
  bottom: 0;
  left: var(--sidebar-width, 240px);
  right: 0;
  z-index: 80;
  background: rgba(17, 24, 39, 0.95);
  backdrop-filter: blur(16px);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 -8px 30px rgba(0, 0, 0, 0.6);
  transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.drawer-header {
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  cursor: pointer;
  user-select: none;
  background: rgba(15, 23, 42, 0.7);
}

.drawer-header:hover {
  background: rgba(30, 41, 59, 0.8);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.8125rem;
  overflow: hidden;
}

.terminal-icon {
  color: var(--accent-cyan, #06b6d4);
  flex-shrink: 0;
}

.header-title {
  font-weight: 600;
  color: #f1f5f9;
  letter-spacing: 0.02em;
}

.log-count {
  background: rgba(255, 255, 255, 0.1);
  color: #94a3b8;
  font-size: 0.7rem;
  padding: 1px 6px;
  border-radius: 999px;
  font-family: var(--font-mono);
}

.latest-summary {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}

.muted {
  color: #64748b;
}

.latest-action {
  color: #cbd5e1;
}

.status-pill {
  font-size: 0.65rem;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  letter-spacing: 0.05em;
}

.status-pill--success {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.status-pill--error {
  background: rgba(244, 63, 94, 0.15);
  color: #f43f5e;
}

.status-pill--running {
  background: rgba(56, 189, 248, 0.15);
  color: #38bdf8;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.icon-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  padding: 4px;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.icon-btn:hover {
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.08);
}

.drawer-body {
  height: 240px;
  overflow-y: auto;
  padding: 12px 16px;
  background: #090d16;
}

.empty-logs {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: #64748b;
  gap: 6px;
}

.empty-icon {
  opacity: 0.4;
  margin-bottom: 4px;
}

.empty-logs p {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 500;
  color: #94a3b8;
}

.empty-hint {
  font-size: 0.75rem;
  color: #475569;
}

.logs-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.log-card {
  background: rgba(17, 24, 39, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-left: 3px solid #64748b;
  border-radius: 6px;
  padding: 8px 12px;
  font-size: 0.75rem;
}

.log-card--success {
  border-left-color: #10b981;
}

.log-card--error {
  border-left-color: #f43f5e;
}

.log-card--running {
  border-left-color: #38bdf8;
}

.log-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.log-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.timestamp {
  color: #64748b;
}

.action-tag {
  color: #38bdf8;
  font-weight: 600;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.65rem;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
}

.status-badge--success {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.status-badge--error {
  background: rgba(244, 63, 94, 0.15);
  color: #f43f5e;
}

.status-badge--running {
  background: rgba(56, 189, 248, 0.15);
  color: #38bdf8;
}

.log-cmd {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #e2e8f0;
  background: rgba(0, 0, 0, 0.4);
  padding: 4px 8px;
  border-radius: 4px;
  overflow-x: auto;
}

.prompt {
  color: #10b981;
  font-weight: bold;
}

.log-output {
  margin-top: 6px;
  padding: 6px 8px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 4px;
  color: #94a3b8;
  max-height: 120px;
  overflow-y: auto;
}

.log-output pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  font-size: 0.7rem;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.font-mono {
  font-family: var(--font-mono, monospace);
}
</style>
