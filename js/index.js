/* eslint-disable no-undef */

import init, {
  search,
  get_index_version as getIndexVersion
} from "../pkg/stork";

import { create, add, clear, setText } from "./dom";
import WasmQueue from "./wasmqueue";

import { generateScoresListItem, generateListItem } from "./pencil";
import { defaultConfig, calculateOverriddenConfig } from "./config";

const prod = process.env.NODE_ENV === "production";
const wasmUrl = prod
  ? "https://files.stork-search.net/stork.wasm"
  : "http://127.0.0.1:8025/stork.wasm";

const wasmQueue = new WasmQueue();
const entities = {};
init(wasmUrl).then(() => {
  wasmQueue.loaded = true;
  wasmQueue.handleWasmLoad();
});

function updateDom(name) {
  if (!name) {
    throw new Error("No name in updateDom call");
  }

  const entity = entities[name];

  if (entity.config.showProgress && entity.progress < 1) {
    if (!entity.elements.progress) {
      entity.elements.progress = create("div", {
        classNames: ["stork-loader"]
      });

      add(entity.elements.progress, "afterend", entity.elements.input);
    }

    entity.elements.progress.style.width = `${entity.progress * 100}%`;
  } else if (entity.elements.progress) {
    entity.elements.progress.style.width = `${entity.progress * 100}%`;
    entity.elements.progress.style.opacity = 0;
  }

  let message = "";

  if (entity.progress < 1) {
    message = "Loading...";
  } else if (entity.query && entity.query.length < 3) {
    message = "...";
  } else if (entity.results) {
    const l = entity.hitCount;
    if (l === 0) {
      message = "No files found.";
    } else if (l === 1) {
      message = "1 file found.";
    } else {
      message = `${l} files found.`;
    }
  }

  if (!entity.elements.message) {
    entity.elements.message = create("div", {
      classNames: ["stork-message"]
    });
    add(entity.elements.message, "afterBegin", entity.elements.output);
  }

  setText(entity.elements.message, message);

  if (entity.results) {
    if (entity.results.length > 0) {
      if (!entity.elements.list) {
        entity.elements.list = create("ul", {
          classNames: ["stork-results"]
        });
        add(entity.elements.list, "beforeEnd", entity.elements.output);
      }

      clear(entity.elements.list);

      entity.results.forEach(result => {
        const listItem = entity.config.showScores
          ? generateScoresListItem(result)
          : generateListItem(result);
        entity.elements.list.insertAdjacentHTML("beforeEnd", listItem);
      });
    } else if (entity.elements.list) {
      entity.elements.output.removeChild(entity.elements.list);
      entity.elements.list = null;
    }
  }

  if (!entity.query || entity.query.length === 0) {
    entity.elements.message = null;
    entity.elements.list = null;
    clear(entity.elements.output);
    entity.elements.output.classList.remove("stork-output-visible");
  } else {
    entity.elements.output.classList.add("stork-output-visible");
  }
}

async function resolveSearch(index, query) {
  if (!wasmQueue.loaded) {
    return null;
  }
  try {
    return JSON.parse(search(index, query));
  } catch (e) {
    // analytics.log(e)
  }
}

function handleInputEvent(event) {
  const name = event.target.getAttribute("data-stork");
  const query = event.target.value;

  entities[name].query = query;

  if (entities[name].index) {
    // eslint-disable-next-line no-use-before-define
    performSearch(name);
  }

  updateDom(name);
}

function performSearch(name) {
  if (!entities[name]) {
    return;
  }

  if (entities[name].elements.input.value) {
    entities[name].query = entities[name].elements.input.value;
  }

  const { query } = entities[name];
  if (query && query.length >= 3) {
    resolveSearch(entities[name].index, query).then(results => {
      // Results might be undefined if resolveSearch errored. However, if this
      // happens, resolveSearch should log the error itself.
      if (!results) {
        return;
      }

      if (process.env.NODE_ENV === "development") {
        console.log(results);
      }

      entities[name].results = results.results;
      entities[name].hitCount = results.total_hit_count;

      // Mutate the result URL, like we do when there's a url prefix or suffix
      const urlPrefix = results.url_prefix || "";
      entities[name].results.map(r => {
        const urlSuffix = r.excerpts[0].fields["_srt_url_suffix"] || "";
        r.entry.url = `${urlPrefix}${r.entry.url}${urlSuffix}`;
      });

      updateDom(name);
    });
  } else {
    entities[name].results = [];
    updateDom(name);
  }
}

function handleDownloadProgress(event, name) {
  const loadedPercentage = event.loaded / event.total;
  entities[name].progress = loadedPercentage;
  if (entities[name].config.showProgress) {
    updateDom(name);
  }
}

function handleLoadedIndex(event, name) {
  const { response } = event.target;
  entities[name].progress = 1;
  entities[name].index = new Uint8Array(response);
  entities[name].indexSize = response.byteLength;

  wasmQueue.runAfterWasmLoaded(() => {
    Object.keys(entities).forEach(key => {
      performSearch(key);
    });
  });

  wasmQueue.runAfterWasmLoaded(() => {
    if (entities[name].config.printIndexInfo) {
      // eslint-disable-next-line no-console
      console.log({
        name,
        sizeInBytes: entities[name].indexSize,
        indexVersion: getIndexVersion(entities[name].index)
      });
    }
  });
}

function loadIndexFromUrl(name, url, callbacks) {
  const r = new XMLHttpRequest();
  r.addEventListener("load", e => {
    if (callbacks.load) {
      callbacks.load(e, name);
    }
  });
  r.addEventListener("progress", e => {
    if (callbacks.progress) {
      callbacks.progress(e, name);
    }
  });
  r.responseType = "arraybuffer";
  r.open("GET", url);
  r.send();
}

export function register(name, url, config = {}) {
  const configOverride = calculateOverriddenConfig(defaultConfig, config);

  entities[name] = { config: configOverride, elements: {} };

  loadIndexFromUrl(name, url, {
    load: (event, indexName) => {
      handleLoadedIndex(event, indexName);
    },
    progress: handleDownloadProgress
  });

  entities[name].elements.input = document.querySelector(
    `input[data-stork=${name}]`
  );
  entities[name].elements.output = document.querySelector(
    `[data-stork=${name}-output]`
  );

  [
    { value: entities[name].elements.input, name: "input element" },
    { value: entities[name].elements.output, name: "output element" }
  ].forEach(element => {
    if (!element.value) {
      throw new Error(
        `Could not register search box "${name}": ${element.name} not found.`
      );
    }
  });

  entities[name].elements.input.addEventListener("input", handleInputEvent);
}

export default {
  register
};
