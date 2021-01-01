import { Entity } from "./entity";
import { Configuration, calculateOverriddenConfig } from "./config";
import { loadIndexFromUrl } from "./indexLoader";
import { wasm_register_index as wasmRegisterIndex } from "stork-search";
import WasmQueue from "./wasmQueue";

export class EntityManager {
  entities: Array<Entity> = [];
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

    this.wasmQueue.runAfterWasmLoaded(() => {
      if (!entity.error) {
        const indexInfo = wasmRegisterIndex(
          entity.name,
          new Uint8Array(response)
        );

        entity.setDownloadProgress(1);
        entity.performSearch(entity.domManager.getQuery());

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
  ): void {
    const fullConfig = calculateOverriddenConfig(config);
    const entity = new Entity(name, url, fullConfig, this.wasmQueue);
    if (this.entities.length < 1) {
      this.entities.push(entity);
    }

    loadIndexFromUrl(entity, url, {
      load: e => this.handleLoadedIndex(entity, e),
      progress: (progress, entity) => {
        entity.setDownloadProgress(progress);
      },
      error: () => {
        entity.setDownloadError();
      }
    });
  }
}
