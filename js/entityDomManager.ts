import { SearchResult } from "../stork-lib/bindings/SearchResult";

import { UIConfig } from "./config";
import { add, clear, create, existsBeyondContainerBounds, setText } from "./dom";
import { EntityDomDelegate } from "./entity";
import { ListItemDisplayOptions, resultToListItem } from "./resultToListItem";
import StorkError from "./storkError";
import { log } from "./util/storkLog";

export default class EntityDomManager {
  name: string;
  config: UIConfig;
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
  private visibleSearchResults: SearchResult[] = [];
  private attachedToDom = false;
  private highlightedResultIndex?: number;
  private message?: string;

  constructor(name: string, delegate: EntityDomDelegate) {
    log(`Creating DomManager for ${name}`);
    this.name = name;
    this.delegate = delegate;
  }

  attach(uiConfig: UIConfig) {
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
    this.config = uiConfig;

    this.list = create("ul", { classNames: ["stork-results"] });
    this.attribution = create("div", { classNames: ["stork-attribution"] });
    this.progressBar = create("div", { classNames: ["stork-progress"] });
    this.messageElem = create("div", { classNames: ["stork-message"] });
    this.closeButton = create("button", { classNames: ["stork-close-button"] });

    this.input.addEventListener("input", () => {
      this.performSearchFromInputValue();
    });

    this.input.addEventListener("keydown", (e) => {
      this.handleKeyDownEvent(e);
    });

    this.attribution.innerHTML = this.config.strings.attribution;
    this.closeButton.innerHTML = this.config.strings.closeButtonSvg;

    this.closeButton.addEventListener("click", () => {
      if (this.input) {
        this.input.value = "";
        this.input.focus();
      }

      this.performSearchFromInputValue();
    });

    add(this.progressBar, "afterend", this.input);

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

  setSearchResults({
    results,
    totalHitCount,
    duration
  }: {
    results: SearchResult[];
    totalHitCount: number;
    duration: number;
  }) {
    this.visibleSearchResults = results;
    this.message = this.config.generateMessage(totalHitCount, duration);
    this.render();
  }

  performSearchFromInputValue() {
    // Stick this in a zero-length setTimeout to get it to run
    // after the event loop ticks
    setTimeout(() => {
      if (!this.searchIsReady() || !this.input) {
        return;
      }

      const query = this.input.value;
      const begin = performance.now();
      const result = this.delegate.performSearch(query);
      const end = performance.now();
      this.config.onQueryUpdate(query);
      if (result.success && result.value) {
        if (query.length < 3) {
          this.visibleSearchResults = [];
          this.message = this.config.strings.queryTooShort;
          this.render();
        } else {
          const { value } = result;
          const { results, total_hit_count } = value;
          this.setSearchResults({ results, totalHitCount: total_hit_count, duration: end - begin });
        }
      } else {
        this.setSearchResults({ results: [], totalHitCount: 0, duration: 0 });
      }
    }, 0);
  }

  handleKeyDownEvent(event: KeyboardEvent): void {
    switch (event.key) {
      case "ArrowDown": {
        if (this.highlightedResultIndex === undefined) {
          this.changeHighlightedResult({ to: 0, shouldScrollTo: true });
        } else {
          const target = Math.min(
            this.highlightedResultIndex + 1,
            this.visibleSearchResults.length - 1
          );
          this.changeHighlightedResult({ to: target, shouldScrollTo: true });
        }
        break;
      }

      case "ArrowUp": {
        if (this.highlightedResultIndex !== undefined) {
          const target = Math.max(0, this.highlightedResultIndex - 1);
          this.changeHighlightedResult({ to: target, shouldScrollTo: true });
        }
        break;
      }

      case "Enter":
        this.selectResult();
        break;

      default:
        return;
    }
  }

  private searchIsReady() {
    return this.indexDownloadProgress === 1 && this.wasmDownloadIsComplete && !this.error;
  }

  private render() {
    if (!this.input || !this.output || !this.list || !this.attribution || !this.messageElem) {
      return;
    }

    this.resetElements();

    if (this.progressBar) {
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

    if (this.input.value.length > 0) {
      if (this.closeButton) {
        add(this.closeButton, "afterend", this.input);
      }

      if (this.message) {
        this.output.classList.add("stork-output-visible");
        add(this.messageElem, "beforeend", this.output);
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
            if (i !== this.highlightedResultIndex) {
              this.changeHighlightedResult({ to: i, shouldScrollTo: false });
            }
          });

          listItem.addEventListener("mouseleave", () => {
            if (i === this.highlightedResultIndex) {
              this.changeHighlightedResult({ to: -1, shouldScrollTo: false });
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
  }

  private changeHighlightedResult(options: { to: number; shouldScrollTo: boolean }): number {
    const previousValue = this.highlightedResultIndex;

    const resolvedIdx = Math.max(
      -1, // `to` will be -1 if we want to clear the highlight
      Math.min(this.visibleSearchResults.length - 1, options.to)
    );

    this.highlightedResultIndex = resolvedIdx;
    const scrollAnchorPoint = (previousValue || 0) < resolvedIdx ? "end" : "start";

    let targetForScrollTo: Element | null = null;

    for (let i = 0; i < this.visibleSearchResults.length; i++) {
      const element = this.list?.children[i];
      if (!element) {
        continue;
      }

      const highlightedClassName = "selected";

      if (i == resolvedIdx) {
        element.classList.add(highlightedClassName);
        targetForScrollTo = element;
      } else {
        element.classList.remove(highlightedClassName);
      }
    }

    if (options.shouldScrollTo) {
      if (targetForScrollTo && this.list) {
        if (existsBeyondContainerBounds(targetForScrollTo as HTMLElement, this.list)) {
          (targetForScrollTo as HTMLElement).scrollIntoView({
            behavior: "smooth",
            block: scrollAnchorPoint,
            inline: "nearest"
          });
        }
      }
    }

    return resolvedIdx;
  }

  private selectResult() {
    if (this.highlightedResultIndex === -1 || this.highlightedResultIndex === undefined) {
      return;
    }

    const result = this.visibleSearchResults[this.highlightedResultIndex];

    console.log(result.entry.url);

    if (this.input) {
      Promise.resolve(this.config.onResultSelected(this.input.value, result));
    }

    const transformedUrl = this.config.transformResultUrl(result.entry.url);

    window.location.assign(transformedUrl);
  }

  private resetElements() {
    if (!this.attachedToDom) {
      return;
    }

    // TODO: Remove event listeners

    clear(this.output);
    clear(this.list);
    this.messageElem?.remove();
    this.closeButton?.remove();
    this.output?.classList.remove("stork-output-visible");
    this.input?.classList.remove("stork-error");
  }
}
