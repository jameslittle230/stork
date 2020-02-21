import init, { search, get_index_version } from "../pkg/stork.js";
import Dom from "./dom.js";
import WasmQueue from "./wasmqueue.js";
const Handlebars = require("handlebars");

Handlebars.registerHelper("highlight", function(
  text,
  queryOffset,
  queryLength
) {
  var queryOffset = parseInt(queryOffset);
  var queryLength = parseInt(queryLength);
  var text = Handlebars.escapeExpression(text);

  return new Handlebars.SafeString(
    [
      text.substring(0, queryOffset),
      '<span class="stork-highlight">',
      text.substring(queryOffset, queryOffset + queryLength),
      "</span>",
      text.substring(queryOffset + queryLength)
    ].join("")
  );
});

// @TODO: Change this URL based on webpack production vs. development
const wasmUrlProduction = "https://files.stork-search.net/stork.wasm";
const _wasmUrlDevelopment = "http://localhost:8888/stork.wasm";
init(wasmUrlProduction).then(() => {
  wasmQueue.loaded = true;
  wasmQueue.handleWasmLoad();
});

var wasmQueue = new WasmQueue();
var entities = {};

const defaultConfig = {
  showProgress: true,
  printIndexInfo: false,

  listItemTemplateString: `
    <li class="stork-result">
      <a href="{{entry.url}}">
          <p class="stork-title">{{entry.title}}</p>
          {{#each result.excerpts}}
            <p class="stork-excerpt">
              ...{{ highlight value query_offset @queryLength}}...
            </p>
          {{/each}}
      </a>
    </li>`
};

function handleDownloadProgress(event, name) {
  let loadedPercentage = event.loaded / event.total;
  entities[name].progress = loadedPercentage;
  if (entities[name].config.showProgress) {
    updateDom(name);
  }
}

function handleLoadedIndex(event, name) {
  let response = event.target.response;
  entities[name].progress = 1;
  entities[name].index = new Uint8Array(response);
  entities[name].indexSize = response.byteLength;

  wasmQueue.runAfterWasmLoaded(function() {
    for (let key in Object.keys(entities)) {
      performSearch(key);
    }
  });

  wasmQueue.runAfterWasmLoaded(function() {
    if (entities[name].config.printIndexInfo) {
      console.log({
        name: name,
        sizeInBytes: entities[name].indexSize,
        indexVersion: get_index_version(entities[name].index)
      });
    }
  });
}

function loadIndexFromUrl(name, url, callbacks) {
  var r = new XMLHttpRequest();
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
  var output = Object.create({});
  for (let key in original) {
    if (key in overrides) {
      output[key] = overrides[key];
    } else {
      output[key] = original[key];
    }
  }
  return output;
}

export function register(name, url, config = {}) {
  let configOverride = calculateOverriddenConfig(defaultConfig, config);
  entities[name] = { config: configOverride, elements: {} };
  entities[name].config.listItemTemplate = Handlebars.compile(
    entities[name].config.listItemTemplateString,
    { strict: true }
  );

  loadIndexFromUrl(name, url, {
    load: (event, name) => {
      handleLoadedIndex(event, name);
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

async function resolveSearch(index, query) {
  if (!wasmQueue.loaded) {
    return null;
  }
  return Array.from(JSON.parse(search(index, query)));
}

function handleInputEvent(event) {
  let name = event.target.getAttribute("data-stork");
  let query = event.target.value;

  entities[name].query = query;

  if (entities[name].index) {
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

  let query = entities[name].query;
  if (query && query.length >= 3) {
    resolveSearch(entities[name]["index"], query).then(results => {
      entities[name].results = results;
      updateDom(name);
    });
  } else {
    entities[name].results = [];
    updateDom(name);
  }
}

function updateDom(name) {
  if (!name) {
    throw new Error("No name in updateDom call");
  }

  let dom = new Dom();
  let entity = entities[name];

  if (entity.config.showProgress && entity.progress < 1) {
    if (!entity.elements.progress) {
      entity.elements.progress = dom.create("div", {
        classNames: ["stork-loader"]
      });

      dom.add(entity.elements.progress, "afterend", entity.elements.input);
    }

    entity.elements.progress.style.width = `${entity.progress * 100}%`;
  } else {
    if (entity.elements.progress) {
      entity.elements.progress.style.width = `${entity.progress * 100}%`;
      entity.elements.progress.style.opacity = 0;
    }
  }

  var message = "";

  if (entity.progress < 1) {
    message = "Loading...";
  } else if (entity.query && entity.query.length < 3) {
    message = "...";
  } else if (entity.results) {
    let l = entity.results.length;
    if (l === 0) {
      message = "No files found.";
    } else if (l === 1) {
      message = "1 file found.";
    } else {
      message = `${l} files found.`;
    }
  }

  if (!entity.elements.message) {
    entity.elements.message = dom.create("div", {
      classNames: ["stork-message"]
    });
    dom.add(entity.elements.message, "afterBegin", entity.elements.output);
  }

  dom.setText(entity.elements.message, message);

  if (entity.results) {
    if (entity.results.length > 0) {
      if (!entity.elements.list) {
        entity.elements.list = dom.create("ul", {
          classNames: ["stork-results"]
        });
        dom.add(entity.elements.list, "beforeEnd", entity.elements.output);
      }

      dom.clear(entity.elements.list);

      for (let result of entity.results) {
        let listItem = entity.config.listItemTemplate(result, {
          data: {
            queryOffset: 8,
            queryLength: entity.query.length
          }
        });
        entity.elements.list.insertAdjacentHTML("beforeEnd", listItem);
      }
    } else {
      if (entity.elements.list) {
        entity.elements.output.removeChild(entity.elements.list);
        entity.elements.list = null;
      }
    }
  }

  if (!entity.query || entity.query.length === 0) {
    entity.elements.message = null;
    entity.elements.list = null;
    dom.clear(entity.elements.output);
    entity.elements.output.classList.remove("stork-output-visible");
  } else {
    entity.elements.output.classList.add("stork-output-visible");
  }
}
