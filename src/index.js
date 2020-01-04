import init, { search } from "../pkg/stork.js";
const Handlebars = require("handlebars");

Handlebars.registerHelper("highlight", function(text, offset) {
  var offset = Handlebars.escapeExpression(offset);
  var text = Handlebars.escapeExpression(text);

  return new Handlebars.SafeString(
    [
      text.substring(0, offset),
      "<span>",
      text.substring(offset),
      "</span>"
    ].join("")
  );
});

// @TODO: Change this URL based on webpack production vs. development
// init("https://d1req3pu7uy8ci.cloudfront.net/stork.wasm");
init("http://localhost:8888/stork.wasm").then(x => {
  wasmLoaded = true;
  for (let key in Object.keys(entities)) {
    performSearch(key);
  }
});

var wasmLoaded = false;
var entities = {};

const defaultConfig = {
  showProgress: true,
  listItemTemplateString: `
    <li class="stork-result">
      <a href="{{entry.path}}">
          <p class="stork-title">{{entry.title}}</p>
          {{#each result.excerpts}}
            <p class="stork-excerpt">
              {{! highlight value query_offset}}
              ...{{value}}...
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
  console.info(`${name}: ${response.byteLength} bytes loaded`);
  entities[name].progress = 1;
  entities[name].index = new Uint8Array(response);

  performSearch(name);
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
    load: handleLoadedIndex,
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
  // entities[name].elements.input.addEventListener("blur", e => {
  //   handleBlurEvent(e);
  // });
}

async function resolveSearch(index, query) {
  if (!wasmLoaded) {
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

// function handleBlurEvent(event) {
//   let name = event.target.getAttribute("data-stork");
//   if (event.target.value == "") {
//     updateDom(name);
//   }
// }

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

class Dom {
  create(name, attributes) {
    let elem = document.createElement(name);
    if (attributes.classNames) {
      elem.setAttribute("class", attributes.classNames.join(" "));
    }
    return elem;
  }

  add(elem, location, reference) {
    reference.insertAdjacentElement(location, elem);
  }

  clear(elem) {
    while (elem.firstChild) {
      elem.removeChild(elem.firstChild);
    }
  }

  setText(elem, text) {
    let textNode = document.createTextNode(text);
    if (elem.firstChild) {
      elem.replaceChild(textNode, elem.firstChild);
    } else {
      elem.appendChild(textNode);
    }
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
      message = "No results found.";
    } else if (l === 1) {
      message = "1 result found.";
    } else {
      message = `${l} results found.`;
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
        let listItem = entity.config.listItemTemplate(result);
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
