import { SearchResult } from "../stork-lib/bindings/SearchResult";

import StorkError from "./storkError";

const defaultRegisterConfig = {
  forceRefreshIndex: false
};

export type RegisterConfiguration = Readonly<typeof defaultRegisterConfig>;

export const resolveRegisterConfig = (object: unknown) => {
  if (object && typeof object !== "object") {
    throw new StorkError(`Your configuration value needs to be an object. Saw a ${typeof object}.`);
  }

  if (!object) {
    return defaultRegisterConfig;
  }

  return {
    ...defaultRegisterConfig,
    ...object
  } as RegisterConfiguration;
};

const CLOSE_BUTTON_SVG = `\
<svg height="0.8em" viewBox="0 0 23 24" xmlns="http://www.w3.org/2000/svg">\
<g fill="none" fill-rule="evenodd" stroke-linecap="round">\
<g transform="translate(-700 -149)" stroke="currentcolor" stroke-width="4">\
<line id="a" x1="702.5" x2="720" y1="152.5" y2="170"/>\
<line transform="translate(711 161) rotate(-90) translate(-711 -161)" x1="702.5" x2="720" y1="152.5" y2="170"/>\
</g>\
</g>\
</svg>`;

const defaultUIConfig = {
  strings: {
    attribution: `Powered by <a href="https://stork-search.net">Stork</a>`,
    closeButtonSvg: CLOSE_BUTTON_SVG,
    queryTooShort: "Searching..."
  },
  generateMessage: (totalResultCount: number, duration: number) => {
    if (totalResultCount === 1) {
      return `${totalResultCount} result in ${duration.toFixed(3)} ms`;
    } else {
      return `${totalResultCount} results in ${duration.toFixed(3)} ms`;
    }
  },
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  onQueryUpdate: (_query: string) => {},
  // eslint-disable-next-line @typescript-eslint/no-empty-function
  onResultSelected: (_query: string, _result: SearchResult) => {},
  transformResultUrl: (url: string) => url
};

export type UIConfig = Readonly<typeof defaultUIConfig>;

export const resolveUIConfig = (object: unknown) => {
  if (object && typeof object !== "object") {
    throw new StorkError(`Your configuration value needs to be an object. Saw a ${typeof object}.`);
  }

  if (!object) {
    return defaultUIConfig;
  }

  return {
    ...defaultUIConfig,
    ...object
  } as UIConfig;
};
