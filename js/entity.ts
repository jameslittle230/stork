import { Configuration } from "./config";
import { Result, SearchData, resolveSearch } from "./searchData";
import { EntityDom, RenderState } from "./entityDom";
import { wasm_register_index } from "stork-search";
import StorkError from "./storkError";

export type EntityState = "initialized" | "loading" | "ready" | "error";

export class Entity {
  readonly name: string;
  readonly url: string;
  readonly config: Configuration;

  private _state: EntityState = "initialized";

  downloadProgress = 0;

  index: Uint8Array;
  results: Array<Result> = [];
  totalResultCount = 0;

  domManager: EntityDom | null;
  eventListenerFunctions: Record<string, (e: Event) => void> = {};
  highlightedResult = 0;
  resultsVisible = false;
  hoverSelectEnabled = true;

  constructor(name: string, url: string, config: Configuration) {
    this.name = name;
    this.url = url;
    this.config = config;
  }

  public get state(): EntityState {
    return this._state;
  }

  public set state(value: EntityState) {
    this._state = value;
    this.render();
  }

  private getCurrentMessage(): string | null {
    if (!this.domManager) return null;
    const query = this.domManager.getQuery();
    if (this.state === "error") {
      return "Error! Check the browser console.";
    } else if (this.state != "ready") {
      return "Loading...";
    } else if (query?.length < this.config.minimumQueryLength) {
      return "Filtering...";
    } else if (this.results) {
      if (this.totalResultCount === 0) {
        return `No ${this.config.resultNoun.plural} found.`;
      } else if (this.totalResultCount === 1) {
        return `1 ${this.config.resultNoun.singular} found.`;
      } else {
        return `${this.totalResultCount} ${this.config.resultNoun.plural} found.`;
      }
    }

    return null;
  }

  private generateRenderConfig(): RenderState {
    return {
      results: this.results,
      resultsVisible: true,
      showScores: this.config.showScores,
      message: this.getCurrentMessage(),
      showProgress: this.config.showProgress,
      progress: this.downloadProgress,
      state: this.state
    };
  }

  private render() {
    if (!this.domManager) return;
    this.domManager.render(this.generateRenderConfig());
  }

  registerIndex(data: Uint8Array): Promise<void> {
    return new Promise((resolve, reject) => {
      const indexInfo = JSON.parse(wasm_register_index(this.name, data));
      if (indexInfo.error) {
        reject(new StorkError(indexInfo.error));
      } else {
        if (this.config.printIndexInfo) {
          console.log(indexInfo);
        }

        this.state = "ready";
        resolve(indexInfo);
      }
    });
  }

  attachToDom(): void {
    this.domManager = new EntityDom(this.name, this);
    this.render();
  }

  injestSearchData(data: SearchData): void {
    this.results = data.results;
    this.totalResultCount = data.total_hit_count;
    this.highlightedResult = 0;

    // Mutate the result URL, like we do when there's a url prefix or suffix
    const urlPrefix = data.url_prefix || "";
    this.results.map(r => {
      let urlSuffix = "";

      const firstInternalAnnotations = r.excerpts
        .map(e => e.internal_annotations)
        .filter(ia => !!ia)[0];

      if (firstInternalAnnotations && firstInternalAnnotations[0]) {
        const annotationMap = firstInternalAnnotations[0];
        if (typeof annotationMap["a"] === "string") {
          urlSuffix += annotationMap["a"];
        }
      }

      // oof
      if (
        r.excerpts &&
        r.excerpts[0] &&
        r.excerpts[0].internal_annotations &&
        r.excerpts[0].internal_annotations[0] &&
        r.excerpts[0].internal_annotations[0]["a"] &&
        typeof r.excerpts[0].internal_annotations[0]["a"] === "string"
      ) {
        urlSuffix = r.excerpts[0].internal_annotations[0]["a"];
      }
      r.entry.url = `${urlPrefix}${r.entry.url}${urlSuffix}`;
    });

    this.render();
  }

  private getSanitizedResults() {
    const results = this.results;
    results.map(result => {
      delete result.title_highlight_ranges;
      result.excerpts.map(excerpt => {
        delete excerpt.highlight_ranges;
        delete excerpt.internal_annotations;
      });
    });
    return results;
  }

  setDownloadProgress = (percentage: number): void => {
    this.state = "loading";
    this.downloadProgress = percentage;
    if (this.config.showProgress) {
      this.render();
    }
  };

  setDownloadError(): void {
    this.state = "error";
  }

  performSearch(query: string): void {
    if (this.state !== "ready") {
      this.render();
      return;
    }

    if (query.length < this.config.minimumQueryLength) {
      this.results = [];
      this.render();
      return;
    }

    try {
      const data = resolveSearch(this.name, query);
      if (!data) return;

      this.injestSearchData(data);

      if (this.config.onQueryUpdate) {
        this.config.onQueryUpdate(query, this.getSanitizedResults());
      }
    } catch (error) {
      console.error(error);
    }
  }
}
