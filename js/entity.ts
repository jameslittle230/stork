import { load_index, perform_search } from "stork-search";

import { Configuration } from "./config";
import EntityDomManager from "./entityDomManager";
import IndexLoader, { IndexLoadValue } from "./indexLoader";
import LoadManager from "./loadManager";
import { Result, SearchValue } from "./searchData";
import { log } from "./util/storkLog";
import WasmLoader from "./wasmLoader";

type WrappedValue<T> = { success: true; value: T } | { success: false };

export type EntityDomDelegate = {
  performSearch: (query: string) => WrappedValue<SearchValue>;
};

export default class Entity implements EntityDomDelegate {
  readonly name: string;
  readonly url: string;
  readonly config: Configuration;
  readonly indexLoadPromise: Promise<IndexLoadValue>;

  private domManager: EntityDomManager;
  private loadManager: LoadManager;
  private indexLoader: IndexLoader;
  private wasmLoader: WasmLoader;

  constructor(
    name: string,
    url: string,
    config: Configuration,
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
    this.domManager = new EntityDomManager(this.name, this.config, this.delegate);
  }

  load(): Promise<IndexLoadValue> {
    this.wasmLoader.runAfterWasmLoaded(`${this.name} entity domManager report success`, () => {
      this.loadManager.setState("wasm", "success");
      this.domManager.setWasmLoadIsComplete(true);
    });

    this.wasmLoader.runAfterWasmError(`${this.name} entity domManager report error`, () => {
      this.loadManager.setState("wasm", "failure");
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
            load_index(this.name, new Uint8Array(buffer));
            log(`Index loaded in WASM, setting loadManager state to success`);
            this.loadManager.setState("index", "success");
          } catch (e) {
            this.loadManager.setState("index", "failure");
            log(
              `Index failed to load in WASM, setting loadManager state to failure. Got error ${e}`
            );
          }
        });
        return buffer;
      })
      .catch((e) => {
        this.loadManager.setState("index", "failure");
        throw e;
      });
  }

  attach() {
    this.domManager.attach();
  }

  performSearch(query: string) {
    if (this.loadManager.getAggregateState() !== "success") {
      log("Returning early from search; not ready yet.");
      return { success: false };
    }

    log(`Performing search for index "${this.name}" with query "${query}"`);
    try {
      return JSON.parse(perform_search(this.name, query));
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

  // public get state(): EntityState {
  //   return this._state;

  // private getCurrentMessage(): string | null {
  //   if (!this.domManager) return null;
  //   const query = this.domManager.getQuery();
  //   if (this.state === "error") {
  //     return "Error! Check the browser console.";
  //   } else if (this.state != "ready") {
  //     return "Loading...";
  //   } else if (query?.length < this.config.minimumQueryLength) {
  //     return "Filtering...";
  //   } else if (this.results) {
  //     if (this.totalResultCount === 0) {
  //       return `No ${this.config.resultNoun.plural} found.`;
  //     } else if (this.totalResultCount === 1) {
  //       return `1 ${this.config.resultNoun.singular} found.`;
  //     } else {
  //       return `${this.totalResultCount} ${this.config.resultNoun.plural} found.`;
  //     }
  //   }

  //   return null;
  // }

  // private generateRenderConfig(): RenderState {
  //   return {
  //     results: this.results,
  //     resultsVisible: true,
  //     showScores: this.config.showScores,
  //     message: this.getCurrentMessage(),
  //     showProgress: this.config.showProgress,
  //     progress: this.downloadProgress,
  //     state: this.state
  //   };
  // }

  // private render() {
  //   if (!this.domManager) return;
  //   this.domManager.render(this.generateRenderConfig());
  // }

  // registerIndex(data: Uint8Array): Promise<void> {
  //   return new Promise((resolve, reject) => {
  //     const indexInfo = JSON.parse(wasm_register_index(this.name, data));
  //     if (indexInfo.error) {
  //       reject(new StorkError(indexInfo.error));
  //     } else {
  //       if (this.config.printIndexInfo) {
  //         console.log(indexInfo);
  //       }

  //       this.state = "ready";
  //       resolve(indexInfo);
  //     }
  //   });
  // }

  // attachToDom(): void {
  //   this.domManager = new EntityDom(this.name, this);
  //   this.render();
  // }

  // injestSearchData(data: SearchData): void {
  //   this.results = data.results;
  //   this.totalResultCount = data.total_hit_count;
  //   this.highlightedResult = 0;

  //   // Mutate the result URL, like we do when there's a url prefix or suffix
  //   const urlPrefix = data.url_prefix || "";
  //   this.results.map(r => {
  //     let urlSuffix = "";

  //     const firstInternalAnnotations = r.excerpts
  //       .map(e => e.internal_annotations)
  //       .filter(ia => !!ia)[0];

  //     if (firstInternalAnnotations && firstInternalAnnotations[0]) {
  //       const annotationMap = firstInternalAnnotations[0];
  //       if (typeof annotationMap["a"] === "string") {
  //         urlSuffix += annotationMap["a"];
  //       }
  //     }

  //     // oof
  //     if (
  //       r.excerpts &&
  //       r.excerpts[0] &&
  //       r.excerpts[0].internal_annotations &&
  //       r.excerpts[0].internal_annotations[0] &&
  //       r.excerpts[0].internal_annotations[0]["a"] &&
  //       typeof r.excerpts[0].internal_annotations[0]["a"] === "string"
  //     ) {
  //       urlSuffix = r.excerpts[0].internal_annotations[0]["a"];
  //     }
  //     r.entry.url = this.config.transformResultUrl(
  //       `${urlPrefix}${r.entry.url}${urlSuffix}`
  //     );
  //   });

  //   this.render();
  // }

  // private getSanitizedResults() {
  //   const results = this.results;
  //   results.map(result => {
  //     delete result.title_highlight_ranges;
  //     result.excerpts.map(excerpt => {
  //       delete excerpt.highlight_ranges;
  //       delete excerpt.internal_annotations;
  //     });
  //   });
  //   return results;
  // }

  // setDownloadProgress = (percentage: number): void => {
  //   this.state = "loading";
  //   this.downloadProgress = percentage;
  //   if (this.config.showProgress) {
  //     this.render();
  //   }
  // };

  // setDownloadError(): void {
  //   this.state = "error";
  // }

  // performSearch(query: string): void {
  //   if (this.state !== "ready") {
  //     this.render();
  //     return;
  //   }

  //   if (query.length < this.config.minimumQueryLength) {
  //     this.results = [];
  //     this.render();
  //     return;
  //   }

  //   try {
  //     const data = resolveSearch(this.name, query);
  //     if (!data) return;

  //     this.injestSearchData(data);

  //     if (this.config.onQueryUpdate) {
  //       this.config.onQueryUpdate(query, this.getSanitizedResults());
  //     }
  //   } catch (error) {
  //     console.error(error);
  //   }
  // }
}
