import { Configuration, calculateOverriddenConfig } from "./config";
import { assert, htmlToElement } from "./util";
import {
  create,
  add,
  clear,
  setText,
  existsBeyondContainerBounds
} from "./dom";
import { generateListItem } from "./pencil";
import { Result, SearchData, resolveSearch } from "./searchData";
import WasmQueue from "./wasmQueue";

interface ElementMap {
  input: Element;
  output: Element;
  progress?: HTMLElement;
  list?: Element;
  message?: Element;
}

export class Entity {
  // immutable
  name: string;
  url: string;
  config: Configuration;
  elements: ElementMap;
  index: Uint8Array;
  wasmQueue: WasmQueue;

  // mutable
  results: Array<Result> = [];
  highlightedResult = 0;
  progress = 0;
  hitCount = 0;
  query = "";
  resultsVisible = false;
  hoverSelectEnabled = true;

  // render options
  scrollAnchorPoint: "start" | "end" = "end";

  constructor(name: string, url: string, configIn: Partial<Configuration>) {
    this.name = name;
    this.url = url;
    this.config = calculateOverriddenConfig(configIn);

    const input = document.querySelector(`input[data-stork=${name}]`);
    const output = document.querySelector(`[data-stork=${name}-output]`);

    assert(
      input !== null,
      `Could not register search box "${name}": input element not found`
    );
    assert(
      output !== null,
      `Could not register search box "${name}": input element not found`
    );

    this.elements = {
      input: input,
      output: output
    };
  }

  private getCurrentMessage(): string | null {
    if (this.progress < 1) {
      return "Loading...";
    } else if (this.query && this.query.length < 3) {
      return "...";
    } else if (this.results) {
      if (this.hitCount === 0) {
        return "No files found.";
      } else if (this.hitCount === 1) {
        return "1 file found.";
      } else {
        return `${this.hitCount} files found.`;
      }
    }

    return null;
  }

  setResultsVisible(val: boolean): void {
    const prev = this.resultsVisible;
    this.resultsVisible = val;

    if (val !== prev) {
      this.render();
    }
  }

  changeHighlightedResult(
    options: Partial<{
      by: number | null;
      to: number | null;
    }>
  ): number {
    const previousValue = this.highlightedResult;

    const intendedIdx = (() => {
      if (typeof options.to === "number") {
        return options.to;
      } else if (typeof options.by === "number") {
        return this.highlightedResult + options.by;
      } else {
        return 0;
      }
    })();

    options.to !== null
      ? options.to
      : this.highlightedResult + (options.by || 0);

    const resolvedIdx = Math.max(
      0,
      Math.min(this.results.length - 1, intendedIdx)
    );

    this.highlightedResult = resolvedIdx;
    this.scrollAnchorPoint = previousValue < resolvedIdx ? "end" : "start";

    let targetForScrollTo = null;

    for (let i = 0; i < this.results.length; i++) {
      const element = this.elements.list?.children[i];
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

    // using options.by as a proxy for keyboard selection
    if (typeof options.by === "number") {
      this.hoverSelectEnabled = false;
      if (targetForScrollTo) {
        if (
          existsBeyondContainerBounds(
            targetForScrollTo as HTMLElement,
            this.elements.list
          )
        ) {
          (targetForScrollTo as HTMLElement).scrollIntoView({
            behavior: "smooth",
            block: this.scrollAnchorPoint,
            inline: "nearest"
          });
        }
      }
    }

    return resolvedIdx;
  }

  injestSearchData(data: SearchData): void {
    this.results = data.results;
    this.hitCount = data.total_hit_count;
    this.highlightedResult = 0;

    // Mutate the result URL, like we do when there's a url prefix or suffix
    const urlPrefix = data.url_prefix || "";
    this.results.map(r => {
      const urlSuffix = r.excerpts[0]?.internal_annotations[0]?.["a"] || "";
      r.entry.url = `${urlPrefix}${r.entry.url}${urlSuffix}`;
    });

    this.render();
  }

  setDownloadProgress(percentage: number): void {
    this.progress = percentage;
    if (this.config.showProgress) {
      this.render();
    }
  }

  /**
   * This method is inherently all side effects since it's manipulating the DOM,
   * but the _only_ side effect should be manipulating the DOM, there should be
   * no other changes to the entity class.
   */
  render(): void {
    // Render progress element if index is downloading
    if (this.config.showProgress && this.progress < 1) {
      if (!this.elements.progress) {
        this.elements.progress = create("div", {
          classNames: ["stork-loader"]
        });

        add(this.elements.progress, "afterend", this.elements.input);
      }

      if (this.elements.progress) {
        this.elements.progress.style.width = `${this.progress * 100}%`;
      }
    } else if (this.elements.progress) {
      this.elements.progress.style.width = `${this.progress * 100}%`;
      this.elements.progress.style.opacity = "0";
    }

    // Render message
    const message = this.getCurrentMessage();
    if (!this.elements.message) {
      this.elements.message = create("div", {
        classNames: ["stork-message"]
      });
      add(this.elements.message, "afterBegin", this.elements.output);
    }
    setText(this.elements.message, message);

    // Render results
    if (this.results?.length > 0 && this.resultsVisible) {
      // Create list if it doesn't exist
      if (!this.elements.list) {
        this.elements.list = create("ul", {
          classNames: ["stork-results"]
        });
        add(this.elements.list, "beforeEnd", this.elements.output);
      }

      clear(this.elements.list);
      this.elements.list?.addEventListener("mousemove", () => {
        this.hoverSelectEnabled = true;
      });

      // Render each result
      for (let i = 0; i < this.results.length; i++) {
        const result = this.results[i];
        const generateOptions = {
          result: result,
          selected: i === this.highlightedResult,
          showScores: this.config.showScores
        };

        const elementToInsert = htmlToElement(
          generateListItem(generateOptions)
        );

        if (elementToInsert) {
          const insertedElement = this.elements.list?.appendChild(
            elementToInsert
          );

          insertedElement?.addEventListener("mousemove", () => {
            if (this.hoverSelectEnabled) {
              if (i !== this.highlightedResult) {
                this.changeHighlightedResult({ by: null, to: i });
              }
            }
          });
        }
      }
    } else if (this.elements.list) {
      this.elements.output.removeChild(this.elements.list);
      delete this.elements.list;
    }

    // Remove output's contents if there's no query
    if (!this.query || this.query.length === 0 || !this.resultsVisible) {
      delete this.elements.message;
      delete this.elements.list;
      clear(this.elements.output);
      this.elements.output.classList.remove("stork-output-visible");
    } else {
      this.elements.output.classList.add("stork-output-visible");
    }
  }

  handleInputEvent(event: Event): void {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    const query = event.srcElement?.value;

    this.query = query;

    if (this.index) {
      // eslint-disable-next-line no-use-before-define
      this.performSearch();
    }

    this.render();
  }

  handleKeyDownEvent(event: KeyboardEvent): void {
    const LEFT = 37;
    const UP = 38;
    const RIGHT = 39;
    const DOWN = 40;
    const RETURN = 13;
    // const SPACE = 32;
    const ESC = 27;

    if (![LEFT, UP, RIGHT, DOWN, RETURN, ESC].includes(event.keyCode)) {
      this.setResultsVisible(true);
      return;
    }

    const resultNodeArray = Array.from(
      this.elements.list?.childNodes || []
    ).filter((n: HTMLElement) => n.className == "stork-result");

    switch (event.keyCode) {
      case DOWN:
        this.changeHighlightedResult({ by: +1 });
        break;

      case UP:
        this.changeHighlightedResult({ by: -1 });
        break;

      case RETURN:
        (Array.from(resultNodeArray[this.highlightedResult].childNodes).filter(
          (n: HTMLElement) => (n as HTMLAnchorElement).href
        )[0] as HTMLAnchorElement).click();
        break;

      case ESC:
        this.setResultsVisible(false);
        break;

      default:
        return;
    }
  }

  performSearch(): void {
    if (!this.wasmQueue.loaded) {
      return;
    }

    if (this.elements.input.nodeValue) {
      this.query = this.elements.input.nodeValue;
    }

    const { query } = this;
    if (query && query.length >= 3) {
      resolveSearch(this.index, query).then((data: SearchData) => {
        if (!data) return;

        if (process.env.NODE_ENV === "development") {
          console.log(data);
        }

        this.injestSearchData(data);
      });
    } else {
      this.results = [];
      this.render();
    }
  }
}
