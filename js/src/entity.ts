import { load_index, perform_search } from "stork-search";

import { IndexStatistics } from "../stork-lib/bindings/IndexStatistics";
import { SearchOutput } from "../stork-lib/bindings/SearchOutput";

import { RegisterConfiguration, UIConfig } from "./config";
import EntityDomManager from "./entityDomManager";
import IndexLoader, { IndexLoadValue } from "./indexLoader";
import LoadManager, { LoadState } from "./loadManager";
import { log } from "./util/storkLog";
import WasmLoader from "./wasmLoader";

type WrappedValue<T> = { success: boolean; value?: T };

export type EntityDomDelegate = {
  performSearch: (query: string) => WrappedValue<SearchOutput>;
};

export default class Entity implements EntityDomDelegate {
  readonly name: string;
  readonly url: string;
  readonly config: RegisterConfiguration;
  readonly indexLoadPromise: Promise<IndexLoadValue>;

  private uiConfig: UIConfig;
  private domManager: EntityDomManager;
  private loadManager: LoadManager;
  private indexLoader: IndexLoader;
  private wasmLoader: WasmLoader;

  indexStatistics: IndexStatistics | null;

  constructor(
    name: string,
    url: string,
    config: RegisterConfiguration,
    indexLoader: IndexLoader,
    wasmLoader: WasmLoader
  ) {
    log(`Entity ${name} constructed`);
    this.name = name;
    this.url = url;
    this.config = config;

    this.indexLoader = indexLoader;
    this.wasmLoader = wasmLoader;
    this.loadManager = new LoadManager(["index", "wasm"]);

    // This needs to exist regardless of whether we've called `attach()` or not,
    // since this is responsible for managing a lot of display state that needs
    // to be kept up-to-date even before we've attached and started displaying
    // that state.
    this.domManager = new EntityDomManager(this.name, this.delegate);
  }

  load(): Promise<IndexLoadValue> {
    this.wasmLoader.runAfterWasmLoaded(`${this.name} entity domManager report success`, () => {
      this.loadManager.setState("wasm", LoadState.Success);
      this.domManager.setWasmLoadIsComplete(true);
    });

    this.wasmLoader.runAfterWasmError(`${this.name} entity domManager report error`, () => {
      this.loadManager.setState("wasm", LoadState.Failure);
      // -> this.loadManager.runOnError
    });

    this.loadManager.runAfterLoad("run entitydom search", () => {
      log(`Index ${this.name} completely loaded, ready for search`);
      this.domManager.performSearchFromInputValue();
    });

    this.loadManager.runOnError("set entitydom to error", () => {
      this.domManager.setError(true);
    });

    return this.indexLoader
      .load(this.url, (percentage) => {
        this.domManager.setProgress(percentage);
      })
      .then((buffer) => {
        this.wasmLoader.runAfterWasmLoaded(`Load index ${this.name}`, () => {
          this.domManager.setProgress(1);
          log(`Index download complete! Got ${buffer.byteLength} bytes`);

          try {
            const indexStatsRaw = load_index(this.name, new Uint8Array(buffer));
            this.indexStatistics = JSON.parse(indexStatsRaw) as IndexStatistics;
            log(`Index loaded in WASM, setting loadManager state to success`);
            log("Index statistics:", this.indexStatistics);
            this.loadManager.setState("index", LoadState.Success);
          } catch (e) {
            this.loadManager.setState("index", LoadState.Failure);
            log(
              `Index failed to load in WASM, setting loadManager state to failure. Got error ${e}`
            );
          }
        });
        return buffer;
      })
      .catch((e) => {
        this.loadManager.setState("index", LoadState.Failure);
        throw e;
      });
  }

  attach(uiConfig: UIConfig) {
    this.uiConfig = uiConfig;
    this.domManager.attach(uiConfig);
  }

  performSearch(query: string) {
    if (this.loadManager.getAggregateState() !== LoadState.Success) {
      log("Returning early from search; not ready yet.");
      return { success: false };
    }

    log(`Performing search for index "${this.name}" with query "${query}"`);

    try {
      const v = perform_search(
        this.name,
        query,
        this.uiConfig?.excerptLength,
        this.uiConfig?.numberOfResults,
        this.uiConfig?.numberOfExcerpts
      );
      const value = JSON.parse(v);
      return {
        success: true,
        value
      };
    } catch (e) {
      log(e);
      return { success: false };
    }
  }

  public get delegate(): EntityDomDelegate {
    return {
      performSearch: (q) => {
        return this.performSearch(q);
      }
    };
  }
}
