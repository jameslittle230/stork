import { Configuration } from "./config";
import { Result, SearchData, resolveSearch } from "./searchData";
import WasmQueue from "./wasmQueue";
import { EntityDom, RenderState } from "./entityDom";

export class Entity {
  readonly name: string;
  readonly url: string;
  readonly config: Configuration;
  readonly wasmQueue: WasmQueue;

  domManager: EntityDom | null;
  eventListenerFunctions: Record<string, (e: Event) => void> = {};
  index: Uint8Array;
  results: Array<Result> = [];
  highlightedResult = 0;
  progress = 0;
  error = false;
  totalResultCount = 0;
  resultsVisible = false;
  hoverSelectEnabled = true;

  constructor(
    name: string,
    url: string,
    config: Configuration,
    wasmQueue: WasmQueue
  ) {
    this.name = name;
    this.url = url;
    this.config = config;
    this.wasmQueue = wasmQueue;
  }

  attachToDom(): void {
    this.domManager = new EntityDom(this.name, this);
  }

  private getCurrentMessage(): string | null {
    if (!this.domManager) return null;
    const query = this.domManager.getQuery();
    if (this.error) {
      return "Error! Check the browser console.";
    } else if (this.progress < 1 || !this.wasmQueue.loaded) {
      return "Loading...";
    } else if (query?.length < this.config.minimumQueryLength) {
      return "Filtering...";
    } else if (this.results) {
      if (this.totalResultCount === 0) {
        return "No files found.";
      } else if (this.totalResultCount === 1) {
        return "1 file found.";
      } else {
        return `${this.totalResultCount} files found.`;
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
      progress: this.progress,
      error: this.error
    };
  }

  private render() {
    if (!this.domManager) return;
    this.domManager.render(this.generateRenderConfig());
  }

  injestSearchData(data: SearchData): void {
    this.results = data.results;
    this.totalResultCount = data.total_hit_count;
    this.highlightedResult = 0;

    // Mutate the result URL, like we do when there's a url prefix or suffix
    const urlPrefix = data.url_prefix || "";
    this.results.map(r => {
      let urlSuffix = "";

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

  setDownloadProgress(percentage: number): void {
    this.error = false;
    this.progress = percentage;
    if (this.config.showProgress) {
      this.render();
    }
  }

  setDownloadError(): void {
    this.progress = 1;
    this.error = true;
    this.render();
  }

  performSearch(query: string): void {
    if (!this.wasmQueue.loaded || this.error) {
      this.render();
      return;
    }

    if (query.length >= this.config.minimumQueryLength) {
      try {
        const data = resolveSearch(this.name, query);
        // .then((data: SearchData) => {
        if (!data) return;

        if (process.env.NODE_ENV === "development") {
          console.log("DEVELOPMENT:", data);
        }

        this.injestSearchData(data);

        if (this.config.onQueryUpdate) {
          this.config.onQueryUpdate(query, this.getSanitizedResults());
        }
      } catch (error) {
        console.error(error);
      }
    } else {
      this.results = [];
      this.render();
    }
  }
}
