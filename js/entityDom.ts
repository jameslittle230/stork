import { Result } from "./searchData";

import {
  create,
  add,
  clear,
  setText,
  existsBeyondContainerBounds
} from "./dom";
import { Entity } from "./entity";
import { ListItemDisplayOptions, resultToListItem } from "./resultToListItem";

interface ElementMap {
  input: HTMLInputElement;
  output: HTMLDivElement;
  progress: HTMLElement;
  list: HTMLElement;
  message: HTMLElement;
  attribution: HTMLElement;
  closeButton: HTMLElement;
}

export interface RenderState {
  results: Array<Result>;
  resultsVisible: boolean;
  showScores: boolean;
  message: string | null;
  showProgress: boolean;
  progress: number | null;
  error: boolean;
}

const hiddenInterfaceRenderState: RenderState = {
  results: [],
  resultsVisible: false,
  showScores: false,
  message: null,
  showProgress: false,
  progress: 1,
  error: false
};

export class EntityDom {
  readonly elements: ElementMap;
  readonly entity: Entity;

  highlightedResult?: number;
  hoverSelectEnabled: boolean;
  lastRenderState: RenderState;

  scrollAnchorPoint: "start" | "end" = "end";

  constructor(name: string, entity: Entity) {
    this.entity = entity;

    const data = [
      {
        selector: `input[data-stork="${name}"]`,
        elementName: "input"
      },
      {
        selector: `div[data-stork="${name}-output"]`,
        elementName: "output"
      }
    ];

    const [input, output] = data.map(d => {
      const element = document.querySelector(d.selector);
      if (!element) {
        throw new Error(
          `Could not register search box "${name}": ${d.elementName} element not found. Make sure an element matches the query selector \`${d.selector}\``
        );
      }

      return element;
    }) as [HTMLInputElement, HTMLDivElement];

    this.elements = {
      input: input,
      output: output,
      list: create("ul", { classNames: ["stork-results"] }),
      attribution: create("div", {
        classNames: ["stork-attribution"]
      }),
      progress: create("div", { classNames: ["stork-progress"] }),
      message: create("div", { classNames: ["stork-message"] }),
      closeButton: create("button", {
        classNames: ["stork-close-button"]
      })
    };

    this.elements.input.addEventListener("input", e => {
      this.handleInputEvent(e as InputEvent);
    });

    this.elements.input.addEventListener("keydown", e => {
      this.handleKeyDownEvent(e as KeyboardEvent);
    });

    this.elements.list?.addEventListener("mousemove", () => {
      this.hoverSelectEnabled = true;
    });

    this.elements.attribution.innerHTML =
      'Powered by <a href="https://stork-search.net">Stork</a>';

    setText(this.elements.closeButton, "×");

    add(this.elements.progress, "afterend", this.elements.input);

    this.elements.closeButton?.addEventListener("click", () => {
      this.elements.input.value = "";
      this.elements.input.focus();
      this.render(hiddenInterfaceRenderState);
    });
  }

  private clearDom() {
    clear(this.elements.output);
    clear(this.elements.list);
    this.elements.closeButton?.remove();
    this.elements.output.classList.remove("stork-output-visible");
  }

  render(state: RenderState): void {
    const query = (this.elements.input as HTMLInputElement).value;
    this.clearDom();
    this.lastRenderState = state;

    if (state.showProgress && state.progress && state.progress < 1) {
      this.elements.progress.style.width = `${state.progress * 100}%`;
    } else if (state.showProgress) {
      this.elements.progress.style.width = `100%`;
      this.elements.progress.style.opacity = "0";
    }

    if (state.error) {
      this.elements.input.classList.add("stork-error");
    }

    if (this.getQuery().length > 0 && state.resultsVisible) {
      this.elements.output.classList.add("stork-output-visible");
      add(this.elements.message, "beforeend", this.elements.output);
    }

    if (state.message) {
      setText(this.elements.message, state.message);
    }

    if (state.results?.length > 0 && state.resultsVisible) {
      add(this.elements.list, "beforeend", this.elements.output);

      for (let i = 0; i < state.results.length; i++) {
        const result = state.results[i];
        const generateOptions: ListItemDisplayOptions = {
          selected: i === this.highlightedResult,
          showScores: state.showScores
        };

        const listItem = resultToListItem(result, generateOptions);
        add(listItem as HTMLElement, "beforeend", this.elements.list);

        listItem.addEventListener("mousemove", () => {
          if (this.hoverSelectEnabled) {
            if (i !== this.highlightedResult) {
              this.changeHighlightedResult({ to: i, shouldScrollTo: false });
            }
          }
        });

        listItem.addEventListener("click", e => {
          e.preventDefault();
          this.selectResult();
        });
      }

      add(this.elements.attribution, "beforeend", this.elements.output);
    }

    if ((query?.length || 0) > 0) {
      add(this.elements.closeButton, "afterend", this.elements.input);
    }
  }

  private selectResult() {
    if (this.highlightedResult != null) {
      const result = this.entity.results[this.highlightedResult];
      if (this.entity.config.onResultSelected) {
        Promise.resolve(
          this.entity.config.onResultSelected(this.getQuery(), result)
        ).finally(() => {
          window.location.assign(result.entry.url);
        });
      } else {
        window.location.assign(result.entry.url);
      }
    }
  }

  changeHighlightedResult(options: {
    to: number;
    shouldScrollTo: boolean;
  }): number {
    const previousValue = this.highlightedResult;

    const resolvedIdx = Math.max(
      0,
      Math.min(this.entity.results.length - 1, options.to)
    );

    this.highlightedResult = resolvedIdx;
    this.scrollAnchorPoint =
      (previousValue || 0) < resolvedIdx ? "end" : "start";

    let targetForScrollTo = null;

    for (let i = 0; i < this.entity.results.length; i++) {
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
    if (options.shouldScrollTo) {
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

  handleKeyDownEvent(event: KeyboardEvent): void {
    const UP = 38;
    const DOWN = 40;
    const RETURN = 13;
    const ESC = 27;

    switch (event.keyCode) {
      case DOWN: {
        if (this.highlightedResult == null) {
          this.changeHighlightedResult({ to: 0, shouldScrollTo: true });
        } else {
          const target = Math.min(
            this.highlightedResult + 1,
            this.entity.results.length - 1
          );
          this.changeHighlightedResult({ to: target, shouldScrollTo: true });
        }
        break;
      }

      case UP: {
        if (this.highlightedResult != null) {
          const target = Math.max(0, this.highlightedResult - 1);
          this.changeHighlightedResult({ to: target, shouldScrollTo: true });
        }
        break;
      }

      case RETURN:
        this.selectResult();
        break;

      case ESC:
        if (!this.lastRenderState.resultsVisible) {
          this.elements.input.value = "";
        }
        this.render(hiddenInterfaceRenderState);
        break;

      default:
        return;
    }
  }

  handleInputEvent(event: InputEvent): void {
    this.entity.performSearch((event.target as HTMLInputElement).value);
  }

  getQuery(): string {
    return (this.elements.input as HTMLInputElement).value;
  }
}
