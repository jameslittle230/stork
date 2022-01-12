import { Result } from "./searchData";

import {
  create,
  add,
  clear,
  setText,
  existsBeyondContainerBounds
} from "./dom";
import { Entity, EntityState } from "./entity";
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
  progress: number;
  state: EntityState;
}

const hiddenInterfaceRenderState: RenderState = {
  results: [],
  resultsVisible: false,
  showScores: false,
  message: null,
  showProgress: false,
  progress: 1,
  state: "ready"
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

    // First, remove saved event listener functions from the element, if they exist.
    // This makes the EntityDom constructor safe to call multiple times, even if
    // the elements on the page haven't changed.
    this.elements.input.removeEventListener(
      "input",
      this.entity.eventListenerFunctions.inputInputEvent
    );

    this.elements.input.removeEventListener(
      "keydown",
      this.entity.eventListenerFunctions.inputKeydownEvent
    );

    // Then, save new event listener functions to the entity so that we can
    // delete those listeners from the corresponding elements when the
    // EntityDom object is recreated.
    this.entity.eventListenerFunctions = {
      inputInputEvent: (e: InputEvent) => {
        this.handleInputEvent(e as InputEvent);
      },

      inputKeydownEvent: (e: KeyboardEvent) => {
        this.handleKeyDownEvent(e as KeyboardEvent);
      }
    };

    // Then, add those newly saved functions as event listeners on the elements.
    this.elements.input.addEventListener(
      "input",
      this.entity.eventListenerFunctions.inputInputEvent
    );

    this.elements.input.addEventListener(
      "keydown",
      this.entity.eventListenerFunctions.inputKeydownEvent
    );

    // We didn't have to do the remove/add dance with this one because
    // this listener behavior is already idempotent.
    this.elements.list?.addEventListener("mousemove", () => {
      this.hoverSelectEnabled = true;
    });

    this.elements.attribution.innerHTML =
      'Powered by <a href="https://stork-search.net">Stork</a>';

    this.elements.closeButton.innerHTML = `
<svg height="0.8em" viewBox="0 0 23 24" xmlns="http://www.w3.org/2000/svg">
<g fill="none" fill-rule="evenodd" stroke-linecap="round">
<g transform="translate(-700 -149)" stroke="currentcolor" stroke-width="4">
<line id="a" x1="702.5" x2="720" y1="152.5" y2="170"/>
<line transform="translate(711 161) rotate(-90) translate(-711 -161)" x1="702.5" x2="720" y1="152.5" y2="170"/>
</g>
</g>
</svg>`;

    if (this.entity.config.showProgress) {
      add(this.elements.progress, "afterend", this.elements.input);
    }

    this.elements.closeButton?.addEventListener("click", () => {
      this.elements.input.value = "";
      this.elements.input.focus();
      this.render(hiddenInterfaceRenderState);
      const [m, n] = [
        this.entity.config.onInputCleared,
        this.entity.config.onResultsHidden
      ];
      m ? m() : null;
      n ? n() : null;
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

    if (state.showProgress) {
      const getFakeProgress = (): number => {
        switch (state.state) {
          case "ready":
          case "error":
            return 1;
          case "initialized":
          case "loading":
            return state.progress * 0.9 + 0.05;
        }
      };

      const progress = getFakeProgress();

      if (progress < 1) {
        this.elements.progress.style.width = `${progress * 100}%`;
        this.elements.progress.style.opacity = "1";
      } else {
        this.elements.progress.style.width = `100%`;
        this.elements.progress.style.opacity = "0";
      }
    }

    if (state.state === "error") {
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

        listItem.addEventListener("mouseleave", () => {
          if (this.hoverSelectEnabled) {
            if (i === this.highlightedResult) {
              this.changeHighlightedResult({ to: -1, shouldScrollTo: false });
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

    if ((query?.length || 0) > 0 && this.entity.config.showCloseButton) {
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
      -1, // `to` will be -1 if we want to clear the highlight
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
        if (this.lastRenderState.resultsVisible) {
          this.render(hiddenInterfaceRenderState);
          const m = this.entity.config.onResultsHidden;
          m ? m() : null;
        } else if (this.elements.input.value.length > 0) {
          this.elements.input.value = "";
          this.render(hiddenInterfaceRenderState); // To clear [x] button
          const m = this.entity.config.onInputCleared;
          m ? m() : null;
        }

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
