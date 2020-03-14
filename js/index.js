/* eslint-disable import/no-named-as-default-member */
/* eslint-disable import/no-named-as-default */
/* eslint-disable import/named */

import init, {
  search,
  get_index_version as getIndexVersion
} from "../pkg/stork";
import Dom from "./dom";
import WasmQueue from "./wasmqueue";

const Handlebars = require("handlebars");

Handlebars.registerHelper(
  "highlight",
  (textUnesc, queryOffsetStr, queryLengthStr) => {
    const queryOffset = parseInt(queryOffsetStr, 10);
    const queryLength = parseInt(queryLengthStr, 10);
    const text = Handlebars.escapeExpression(textUnesc);

    return new Handlebars.SafeString(
      [
        text.substring(0, queryOffset),
        '<span class="stork-highlight">',
        text.substring(queryOffset, queryOffset + queryLength),
        "</span>",
        text.substring(queryOffset + queryLength)
      ].join("")
    );
  }
);

const prod = process.env.NODE_ENV === "production";
const wasmUrl = prod
  ? "https://files.stork-search.net/stork.wasm"
  : "http://127.0.0.1:8080/stork.wasm";

const showScoresListItemTemplateString = `
    <li class="stork-result">
      <a href="{{entry.url}}">
          <div style="display: flex; justify-content: space-between">
            <p class="stork-title">{{entry.title}}</p>
            <code><b>{{score}}</b></code>
          </div>
          
          {{#each excerpts}}
            <div style="display: flex; justify-content: space-between">
              <p class="stork-excerpt">
              ...{{ highlight text highlight_char_offset @queryLength}}...
              </p>
              <code>{{score}}</code>
            </div>
          {{/each}}
      </a>
    </li>`;

const wasmQueue = new WasmQueue();
const entities = {};
init(wasmUrl).then(() => {
  wasmQueue.loaded = true;
  wasmQueue.handleWasmLoad();
});

const defaultConfig = {
  showProgress: true,
  printIndexInfo: false,
  showScores: false,

  listItemTemplateString: `
    <li class="stork-result">
      <a href="{{entry.url}}">
          <p class="stork-title">{{entry.title}}</p>
          {{#each excerpts}}
            <p class="stork-excerpt">
              ...{{ highlight text highlight_char_offset @queryLength}}...
            </p>
          {{/each}}
      </a>
    </li>`
};

function updateDom(name) {
  if (!name) {
    throw new Error("No name in updateDom call");
  }

  const entity = entities[name];

  if (entity.config.showProgress && entity.progress < 1) {
    if (!entity.elements.progress) {
      entity.elements.progress = Dom.create("div", {
        classNames: ["stork-loader"]
      });

      Dom.add(entity.elements.progress, "afterend", entity.elements.input);
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
    entity.elements.message = Dom.create("div", {
      classNames: ["stork-message"]
    });
    Dom.add(entity.elements.message, "afterBegin", entity.elements.output);
  }

  Dom.setText(entity.elements.message, message);

  if (entity.results) {
    if (entity.results.length > 0) {
      if (!entity.elements.list) {
        entity.elements.list = Dom.create("ul", {
          classNames: ["stork-results"]
        });
        Dom.add(entity.elements.list, "beforeEnd", entity.elements.output);
      }

      Dom.clear(entity.elements.list);

      entity.results.forEach(result => {
        const listItem = entity.config.listItemTemplate(result, {
          data: {
            queryOffset: 8,
            queryLength: entity.query.length
          }
        });
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
    Dom.clear(entity.elements.output);
    entity.elements.output.classList.remove("stork-output-visible");
  } else {
    entity.elements.output.classList.add("stork-output-visible");
  }
}

async function resolveSearch(index, query) {
  if (!wasmQueue.loaded) {
    return null;
  }
  return JSON.parse(search(index, query));
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
      entities[name].results = results.results;
      entities[name].hitCount = results.total_hit_count;
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

function calculateOverriddenConfig(original, overrides) {
  const output = Object.create({});
  Object.keys(original).forEach(key => {
    if (key in overrides) {
      output[key] = overrides[key];
    } else {
      output[key] = original[key];
    }
  });
  return output;
}

export function register(name, url, config = {}) {
  const configOverride = calculateOverriddenConfig(defaultConfig, config);

  // Use the showScores list item template string if the showScores config key
  // is set to true.
  if (
    configOverride.showScores &&
    configOverride.listItemTemplateString ===
      defaultConfig.listItemTemplateString
  ) {
    configOverride.listItemTemplateString = showScoresListItemTemplateString;
  }

  entities[name] = { config: configOverride, elements: {} };
  entities[name].config.listItemTemplate = Handlebars.compile(
    entities[name].config.listItemTemplateString,
    { strict: true }
  );

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
