import { Configuration } from "./config";
import Entity from "./entity";
import StorkError from "./storkError";

export default class EntityStore {
  store: Record<string, Entity> = {};

  insert(name: string, entity: Entity, config: Configuration) {
    if (this.store[name] && !config.forceRefreshIndex) {
      throw new StorkError(
        "Called downloadIndex() with an identifier that already exists. Did you mean to set forceRefreshIndex to true?"
      );
    }

    this.store[name] = entity;
  }

  get(name: string) {
    if (!this.store[name]) {
      throw new StorkError(`No index ${name} found in store`);
    }
    return this.store[name];
  }

  debug() {
    return {
      store: this.store
    };
  }
}
