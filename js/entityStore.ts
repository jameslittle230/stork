import { Configuration } from "./config";
import Entity from "./entity";
import StorkError from "./storkError";

export default class EntityStore {
  store: Record<string, Entity> = {};

  insert(name: string, entity: Entity, _config: Configuration) {
    // TODO: Handle user-configurable overwrite
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
