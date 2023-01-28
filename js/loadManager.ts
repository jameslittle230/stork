import StorkError from "./storkError";
import { log } from "./util/storkLog";

export enum LoadState {
  NotStarted,
  Incomplete,
  Success,
  Failure
}

export default class LoadManager {
  components: Record<string, LoadState> = {};

  private queue: { name: string; fn: () => void }[] = [];
  private errorQueue: { name: string; fn: () => void }[] = [];

  constructor(components: string[]) {
    components.forEach((component) => {
      this.components[component] = LoadState.NotStarted;
    });
  }

  setState(component: string, state: LoadState) {
    if (this.components[component] === undefined) {
      throw new Error(`Could not find component ${component} in load manager`);
    }

    if (this.components[component] === LoadState.Success && state !== LoadState.Success) {
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
      case LoadState.Failure:
        this.flushErrorQueue();
        return;
      case LoadState.Success:
        this.flushQueue();
        return;
    }
  }

  getState(component: string): LoadState {
    return this.components[component];
  }

  getAggregateState(): LoadState {
    const values = Object.values(this.components);
    if (values.every((v) => v === LoadState.Success)) {
      return LoadState.Success;
    } else if (values.every((v) => v === LoadState.NotStarted)) {
      return LoadState.NotStarted;
    } else if (values.includes(LoadState.NotStarted)) {
      return LoadState.NotStarted;
    }
    return LoadState.Incomplete;
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
