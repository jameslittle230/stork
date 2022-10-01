import { Configuration } from "./config";
import { add, clear, create } from "./dom";
import { EntityDomDelegate } from "./entity";
import StorkError from "./storkError";
import { log } from "./util/storkLog";

export default class EntityDomManager {
  name: string;
  config: Configuration;
  delegate: EntityDomDelegate;

  input: HTMLInputElement;
  output: HTMLDivElement;

  list: HTMLElement;
  attribution: HTMLElement;
  progressBar: HTMLElement;
  message: HTMLElement;
  closeButton: HTMLElement;

  private error = false;
  private indexDownloadProgress = 0;
  private wasmDownloadIsComplete = false;
  private visibleSearchResults: object[] = [];
  private attachedToDom = false;

  constructor(name: string, config: Configuration, delegate: EntityDomDelegate) {
    log(`Creating DomManager for ${name}`);
    this.name = name;
    this.config = config;
    this.delegate = delegate;
  }

  attach() {
    const input = document.querySelector(`input[data-stork="${this.name}"]`);
    const output = document.querySelector(`input[data-stork="${this.name}"]`);

    console.error(38, input, output);

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
    this.message = create("div", { classNames: ["stork-message"] });
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

  setSearchResults(results: object[]) {
    this.visibleSearchResults = results;
    this.render();
  }

  performSearchFromInputValue() {
    if (!this.searchIsReady()) {
      return;
    }

    const query = this.input.value;
    const searchResults = this.delegate.performSearch(query);
    this.setSearchResults(searchResults);
  }

  private searchIsReady() {
    return this.indexDownloadProgress === 1 && this.wasmDownloadIsComplete && !this.error;
  }

  private render() {
    if (!this.attachedToDom) {
      return;
    }

    this.resetElements();

    if (this.config.showProgress) {
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
  }

  private resetElements() {
    if (!this.attachedToDom) {
      return;
    }

    clear(this.output);
    clear(this.list);
    this.closeButton?.remove();
    this.output.classList.remove("stork-output-visible");
    this.input.classList.remove("stork-error");
  }
}
