let logs: { timestamp: Date; message: any[] }[] = [];

export function log(...args: any[]) {
  logs.push({ timestamp: new Date(), message: args });
  if (logs.length > 200) {
    logs = logs.slice(1);
  }

  // @ts-ignore
  if (window.__stork_debug_log === 1) {
    console.log("%cStorkLog:", "color:green", ...args);
  }
}

export function getDebugLogs() {
  return logs;
}
