import { Configuration } from "./config";
import { add, create } from "./dom";
import StorkError from "./storkError";

export default class EntityDomManager {
  name: string;
  config: Configuration;

  input: HTMLInputElement;
  output: HTMLDivElement;

  list: HTMLElement;
  attribution: HTMLElement;
  progressBar: HTMLElement;
  message: HTMLElement;
  closeButton: HTMLElement;

  private indexDownloadProgress = 0;
  private wasmLoadState: "incomplete" | "success" | "failure" = "incomplete";

  constructor(name: string, config: Configuration) {
    this.name = name;
    this.config = config;
  }

  attach() {
    const input = document.querySelector(`input[data-stork="${this.name}"]`);
    const output = document.querySelector(`input[data-stork="${this.name}"]`);

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

    // TODO: eventlistener stuff

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
  }

  setProgress(number: number) {
    this.indexDownloadProgress = number;
    this.render();
  }

  setWasmLoadState(state: typeof this.wasmLoadState) {
    this.wasmLoadState = state;
    this.render();
  }

  private render() {
    if (this.config.showProgress) {
      const getFakeProgress = (): number => {
        switch (this.wasmLoadState) {
          case "failure":
            return 1;
          case "success":
            return this.indexDownloadProgress;
          case "incomplete":
            return this.indexDownloadProgress * 0.9 + 0.05;
        }
      };

      const progress = getFakeProgress();

      if (progress < 1) {
        this.progressBar.style.width = `${progress * 100}%`;
        this.progressBar.style.opacity = "1";
      } else {
        this.progressBar.style.width = `100%`;
        this.progressBar.style.opacity = "0";
      }

      switch (this.wasmLoadState) {
        case "incomplete":
          this.progressBar.style.backgroundColor = `gray`;
          break;
        case "success":
          this.progressBar.style.removeProperty("background-color");
          break;
      }
    }
  }
}
