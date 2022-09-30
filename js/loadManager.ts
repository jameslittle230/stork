import { log } from "./util/storkLog";

export type LoadState = "incomplete" | "success" | "failure";

export default class LoadManager {
  components: Record<string, LoadState> = {};

  private queue: { name: string; fn: () => void }[] = [];
  private errorQueue: { name: string; fn: () => void }[] = [];

  constructor(components: string[]) {
    components.forEach((component) => {
      this.components[component] = "incomplete";
    });
  }

  setState(component: string, state: LoadState) {
    if (!this.components[component]) {
      throw new Error("");
    }

    if (this.components[component] === state) {
      log(`Tried setting ${component} to ${state} in LoadManager, but value is already set`);
      return;
    }

    if (this.components[component] !== "incomplete") {
      throw new Error(
        "Tried to set state to a final value when it's already set to a different final value"
      );
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
      fn();
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
