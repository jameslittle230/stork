import { Entity } from "./entity";
import {
  Configuration,
  calculateOverriddenConfig,
  defaultConfig
} from "./config";
import { loadIndexFromUrl } from "./indexLoader";
import { wasm_register_index as wasmRegisterIndex } from "stork-search";
import WasmQueue from "./wasmQueue";

export class EntityManager {
  entities: Record<string, Entity> = {};
  wasmQueue: WasmQueue;

  constructor(wasmQueue: WasmQueue) {
    this.wasmQueue = wasmQueue;
  }

  handleLoadedIndex(entity: Entity, event: Event): void {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    const { status, response } = (event as ProgressEvent<
      XMLHttpRequestEventTarget
    >).target;

    if (status < 200 || status > 299) {
      entity.setDownloadError();
      throw new Error(`Got a ${status} error from ${entity.url}!`);
    }

    if (!this.wasmQueue) {
      throw new Error("Internal Error - this.wasmQueue doesn't exist");
    }

    this.wasmQueue.runAfterWasmLoaded(() => {
      if (!entity.error) {
        const indexInfo = wasmRegisterIndex(
          entity.name,
          new Uint8Array(response)
        );

        entity.setDownloadProgress(1);

        if (entity.config.printIndexInfo) {
          // eslint-disable-next-line no-console
          console.log({
            name: entity.name,
            sizeInBytes: response.byteLength,
            ...JSON.parse(indexInfo)
          });
        }
      }
    });
  }

  public register(
    name: string,
    url: string,
    config: Partial<Configuration>
  ): Promise<void> {
    return new Promise((res, rej) => {
      let fullConfig = defaultConfig;
      try {
        fullConfig = calculateOverriddenConfig(config);
      } catch (error) {
        rej(error);
        return;
      }

      if (this.entities[name]) {
        // TODO: Add a config option to turn this off, if overwriting an index
        // is expected behavior for you
        console.warn(
          `Search index with name ${name} already exists! Overwriting.`
        );
      }

      if (!this.wasmQueue) {
        rej(new Error("Internal Stork error"));
        return;
      }

      const entity = new Entity(name, url, fullConfig, this.wasmQueue);
      this.entities[name] = entity;

      loadIndexFromUrl(entity, url, {
        load: e => this.handleLoadedIndex(entity, e),
        progress: (progress, entity) => {
          entity.setDownloadProgress(progress);
        },
        error: () => {
          entity.setDownloadError();
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
