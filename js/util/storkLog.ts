let logs: { timestamp: Date; message: unknown[] }[] = [];

export function log(...args: unknown[]) {
  logs.push({ timestamp: new Date(), message: args });
  if (logs.length > 200) {
    logs = logs.slice(1);
  }

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (window && window.__stork_debug_log === 1) {
    console.log("%cStorkLog:", "color:green", ...args);
  }
}

export function getDebugLogs() {
  return logs;
}
