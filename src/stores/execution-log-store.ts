import { defineStore } from 'pinia';

export interface ExecutionLogEntry {
  id: string;
  timestamp: string;
  action: string;
  command: string;
  status: 'running' | 'success' | 'error';
  output: string;
  durationMs?: number;
}

export const useExecutionLogStore = defineStore('executionLog', {
  state: () => ({
    logs: [] as ExecutionLogEntry[],
    drawerOpen: false,
  }),
  getters: {
    hasLogs(state): boolean {
      return state.logs.length > 0;
    },
    latestLog(state): ExecutionLogEntry | null {
      return state.logs.length > 0 ? state.logs[0] : null;
    },
  },
  actions: {
    addLog(action: string, command: string, status: 'running' | 'success' | 'error' = 'running', output = ''): string {
      const now = new Date();
      const timeStr = now.toTimeString().split(' ')[0] + '.' + String(now.getMilliseconds()).padStart(3, '0');
      const id = `${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
      
      this.logs.unshift({
        id,
        timestamp: timeStr,
        action,
        command,
        status,
        output,
      });

      // Keep maximum 50 entries
      if (this.logs.length > 50) {
        this.logs.pop();
      }

      return id;
    },
    updateLog(id: string, updates: Partial<ExecutionLogEntry>) {
      const index = this.logs.findIndex((l) => l.id === id);
      if (index !== -1) {
        this.logs[index] = { ...this.logs[index], ...updates };
      }
    },
    clearLogs() {
      this.logs = [];
    },
    toggleDrawer() {
      this.drawerOpen = !this.drawerOpen;
    },
    openDrawer() {
      this.drawerOpen = true;
    },
    closeDrawer() {
      this.drawerOpen = false;
    },
  },
});

