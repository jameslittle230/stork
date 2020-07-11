import { Entity } from "./entity";
import { Configuration } from "./config";
import { loadIndexFromUrl } from "./indexLoader";
import { get_index_version as getIndexVersion } from "../pkg/stork";
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
    const { response } = (event as ProgressEvent<
      XMLHttpRequestEventTarget
    >).target;

    const indexSize = response.byteLength;
    entity.setDownloadProgress(1);
    entity.index = new Uint8Array(response);

    this.wasmQueue.runAfterWasmLoaded(() => {
      entity.performSearch();
    });

    if (entity.config.printIndexInfo) {
      this.wasmQueue.runAfterWasmLoaded(() => {
        // eslint-disable-next-line no-console
        console.log({
          name: entity.name,
          sizeInBytes: indexSize,
          indexVersion: getIndexVersion(entity.index)
        });
      });
    }
  }

  public register(
    name: string,
    url: string,
    config: Partial<Configuration>
  ): void {
    const entity = new Entity(name, url, config);
    this.entities[name] = entity;
    entity.wasmQueue = this.wasmQueue;

    loadIndexFromUrl(entity, url, {
      load: e => this.handleLoadedIndex(entity, e),
      progress: (progress, entity) => {
        entity.setDownloadProgress(progress);
      }
    });

    entity.elements.input.addEventListener("input", e => {
      entity.handleInputEvent(e);
    });

    /**
     * Handle non-input keypresses for the input field, e.g. arrow keys.
     * (keypress event doesn't work here)
     */
    entity.elements.input.addEventListener("keydown", e => {
      entity.handleKeyDownEvent(e as KeyboardEvent);
    });
  }
}
