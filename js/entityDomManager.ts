import { Configuration } from "./config";
import { add, clear, create, setText } from "./dom";
import { EntityDomDelegate } from "./entity";
import { ListItemDisplayOptions, resultToListItem } from "./resultToListItem";
import { Result } from "./searchData";
import StorkError from "./storkError";
import { log } from "./util/storkLog";

export default class EntityDomManager {
  name: string;
  config: Configuration;
  delegate: EntityDomDelegate;

  input: HTMLInputElement | null;
  output: HTMLDivElement | null;

  list: HTMLElement | null;
  attribution: HTMLElement | null;
  progressBar: HTMLElement | null;
  messageElem: HTMLElement | null;
  closeButton: HTMLElement | null;

  private error = false;
  private indexDownloadProgress = 0;
  private wasmDownloadIsComplete = false;
  private visibleSearchResults: Result[] = [];
  private attachedToDom = false;
  private highlightedResultIndex?: number;
  private hoverSelectEnabled = false;
  private message?: string;

  constructor(name: string, config: Configuration, delegate: EntityDomDelegate) {
    log(`Creating DomManager for ${name}`);
    this.name = name;
    this.config = config;
    this.delegate = delegate;
  }

  attach() {
    const input = document.querySelector(`input[data-stork="${this.name}"]`);
    const output = document.querySelector(`div[data-stork="${this.name}-output"]`);

    if (!input) {
      throw new StorkError("No input element found");
    }

    if (!output) {
      throw new StorkError("No output element found");
    }

    this.input = input as HTMLInputElement;
    this.output = output as HTMLDivElement;

    this.list = create("ul", { classNames: ["stork-results"] });
    this.attribution = create("div", { classNames: ["stork-attribution"] });
    this.progressBar = create("div", { classNames: ["stork-progress"] });
    this.messageElem = create("div", { classNames: ["stork-message"] });
    this.closeButton = create("button", { classNames: ["stork-close-button"] });

    this.input.addEventListener("input", () => {
      this.performSearchFromInputValue();
    });

    this.attribution.innerHTML = 'Powered by <a href="https://stork-search.net">Stork</a>';

    this.closeButton.innerHTML = `
<svg height="0.8em" viewBox="0 0 23 24" xmlns="http://www.w3.org/2000/svg">
<g fill="none" fill-rule="evenodd" stroke-linecap="round">
<g transform="translate(-700 -149)" stroke="currentcolor" stroke-width="4">
<line id="a" x1="702.5" x2="720" y1="152.5" y2="170"/>
<line transform="translate(711 161) rotate(-90) translate(-711 -161)" x1="702.5" x2="720" y1="152.5" y2="170"/>
</g>
</g>
</svg>`;

    if (this.config?.showProgress) {
      add(this.progressBar, "afterend", this.input);
    }

    this.attachedToDom = true;

    this.resetElements();
  }

  setProgress(number: number) {
    this.indexDownloadProgress = number;
    this.render();
  }

  setWasmLoadIsComplete(state: boolean) {
    this.wasmDownloadIsComplete = state;
    this.render();
  }

  setError(state: boolean) {
    this.error = state;
    this.render();
  }

  setSearchResults(results: Result[], totalHitCount: number, duration: number) {
    this.visibleSearchResults = results;
    this.message = `${totalHitCount} results in ${duration} ms`;
    this.render();
  }

  performSearchFromInputValue() {
    if (!this.searchIsReady() || !this.input) {
      return;
    }

    const query = this.input.value;
    const begin = performance.now();
    console.time("search");
    const result = this.delegate.performSearch(query);
    const end = performance.now();
    console.timeEnd("search");
    if (result.success) {
      const { value } = result;
      const { results, total_hit_count } = value;
      this.setSearchResults(results, total_hit_count, end - begin);
    } else {
      this.setSearchResults([], 0, 0);
    }
    // const { results, url_prefix } = value;
  }

  private searchIsReady() {
    return this.indexDownloadProgress === 1 && this.wasmDownloadIsComplete && !this.error;
  }

  private render() {
    if (!this.input || !this.output || !this.list || !this.attribution || !this.messageElem) {
      return;
    }

    this.resetElements();

    if (this.config.showProgress && this.progressBar) {
      const getFakeProgress = (): number => {
        if (this.error) {
          return 1;
        }

        if (!this.wasmDownloadIsComplete) {
          return this.indexDownloadProgress * 0.95 + 0.01;
        }

        return this.indexDownloadProgress * 0.95 + 0.05;
      };

      const progress = getFakeProgress();

      if (this.error) {
        this.input.classList.add("stork-error");
      }

      if (progress < 1) {
        this.progressBar.style.width = `${progress * 100}%`;
        this.progressBar.style.opacity = "1";
      } else {
        this.progressBar.style.width = `100%`;
        this.progressBar.style.opacity = "0";
      }

      if (!this.wasmDownloadIsComplete) {
        this.progressBar.style.backgroundColor = `gray`;
      } else {
        this.progressBar.style.removeProperty("background-color");
      }
    }

    if (this.message) {
      this.output.classList.add("stork-output-visible");
      add(this.messageElem, "beforeend", this.output);
      console.log(this.message);
      setText(this.messageElem, this.message);
    }

    if (this.visibleSearchResults.length > 0) {
      this.output.classList.add("stork-output-visible");
      add(this.list, "beforeend", this.output);

      for (let i = 0; i < this.visibleSearchResults.length; i++) {
        const result = this.visibleSearchResults[i];
        const generateOptions: ListItemDisplayOptions = {
          selected: i === this.highlightedResultIndex,
          showScores: false
        };

        const listItem = resultToListItem(result, generateOptions);
        add(listItem as HTMLElement, "beforeend", this.list);

        listItem.addEventListener("mousemove", () => {
          if (this.hoverSelectEnabled) {
            if (i !== this.highlightedResultIndex) {
              this.changeHighlightedResult({ to: i, shouldScrollTo: false });
            }
          }
        });

        listItem.addEventListener("mouseleave", () => {
          if (this.hoverSelectEnabled) {
            if (i === this.highlightedResultIndex) {
              this.changeHighlightedResult({ to: -1, shouldScrollTo: false });
            }
          }
        });

        listItem.addEventListener("click", (e) => {
          e.preventDefault();
          this.selectResult();
        });
      }

      add(this.attribution, "beforeend", this.output);
    }
  }

  private changeHighlightedResult(options: { to: number; shouldScrollTo: boolean }): number {
    return options.to;
  }

  private selectResult() {
    if (!this.highlightedResultIndex) {
      return;
    }

    const result = this.visibleSearchResults[this.highlightedResultIndex];

    // todo: call user config function

    window.location.assign(result.entry.url);
  }

  private resetElements() {
    if (!this.attachedToDom) {
      return;
    }

    clear(this.output);
    clear(this.list);
    this.messageElem?.remove();
    this.closeButton?.remove();
    this.output?.classList.remove("stork-output-visible");
    this.input?.classList.remove("stork-error");
  }
}
