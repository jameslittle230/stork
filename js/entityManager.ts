import { Entity } from "./entity";
import { Configuration, calculateOverriddenConfig } from "./config";
import { loadIndexFromUrl } from "./loaders/indexLoader";
import WasmQueue from "./wasmQueue";
import StorkError from "./storkError";

export class EntityManager {
  entities: Record<string, Entity> = {};
  wasmQueue: WasmQueue;

  constructor(wasmQueue: WasmQueue) {
    this.wasmQueue = wasmQueue;
  }

  public register(
    name: string,
    url: string,
    partialConfig: Partial<Configuration>
  ): Promise<void> {
    return new Promise((res, rej) => {
      const fullConfig = calculateOverriddenConfig(partialConfig);
      if (fullConfig instanceof StorkError) {
        rej(fullConfig);
        return;
      }

<<<<<<< HEAD
      if (this.entities[name] && !fullConfig.forceOverwrite) {
        throw new StorkError(
          `You're registering an index named \`${name}\`, but that already exists. If this is expected, set forceOverwrite to true in your Javascript config to allow overwriting indexes.`
=======
      if (this.entities[name]) {
        // @TODO: Add a config option to turn this off, if overwriting an index
        // is expected behavior for you
        console.warn(
          `Search index with name ${name} already exists! Overwriting.`
>>>>>>> e4baf2c... todo comments
        );
      }

      const entity = new Entity(name, url, fullConfig);
      this.entities[name] = entity;

      loadIndexFromUrl(url, {
        progress: percentage => {
          entity.setDownloadProgress(percentage);
        },

        load: response => {
          this.wasmQueue.runAfterWasmLoaded(() => {
            entity.registerIndex(new Uint8Array(response)).then(res).catch(rej);
          });
        },

        error: () => {
          entity.setDownloadError();
          rej();
        }
      });
    });
  }

  public attachToDom(name: string): void {
    if (!this.entities[name]) {
      throw new Error(`Index ${name} has not been registered!`);
    }

    this.entities[name].attachToDom();
  }
}
