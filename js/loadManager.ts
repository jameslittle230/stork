import StorkError from "./storkError";
import { log } from "./util/storkLog";

export type LoadState = "notStarted" | "incomplete" | "success" | "failure";

export default class LoadManager {
  components: Record<string, LoadState> = {};

  private queue: { name: string; fn: () => void }[] = [];
  private errorQueue: { name: string; fn: () => void }[] = [];

  constructor(components: string[]) {
    components.forEach((component) => {
      this.components[component] = "notStarted";
    });
  }

  setState(component: string, state: LoadState) {
    if (!this.components[component]) {
      throw new Error("");
    }

    if (this.components[component] === "success" && state !== "success") {
      throw new StorkError(
        `Tried to set ${component} to "${state}" after it's already been set to "success", which is a final value`
      );
    }

    if (this.components[component] === state) {
      log(
        `Tried setting ${component} to "${state}" in LoadManager, but ${component}'s state is already "${state}"`
      );
      return;
    }

    log(`Setting ${component} to ${state} in LoadManager`);

    this.components[component] = state;

    switch (this.getAggregateState()) {
      case "failure":
        this.flushErrorQueue();
        return;
      case "success":
        this.flushQueue();
        return;
    }
  }

  getState(component: string): LoadState {
    return this.components[component];
  }

  getAggregateState(): LoadState {
    const values = Object.values(this.components);
    if (values.every((v) => v === "success")) {
      return "success";
    } else if (values.every((v) => v === "notStarted")) {
      return "notStarted";
    } else if (values.includes("failure")) {
      return "failure";
    }
    return "incomplete";
  }

  runAfterLoad(debugName: string, fn: () => void) {
    this.queue.push({ name: debugName, fn });
  }

  runOnError(debugName: string, fn: () => void) {
    this.errorQueue.push({ name: debugName, fn });
  }

  private flushQueue() {
    log(`Flushing load function queue (${this.queue.length} functions)`);
    this.queue.forEach(({ name, fn }) => {
      log(`Running load function "${name}"`);
      try {
        fn();
      } catch (_e) {
        // no-op
      }
    });
    this.queue = [];
  }

  private flushErrorQueue() {
    log(`Flushing load function queue (${this.queue.length} functions)`);
    this.errorQueue.forEach(({ fn }) => {
      fn();
    });
    this.errorQueue = [];
  }
}
