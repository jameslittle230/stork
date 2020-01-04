import init, { search } from "../pkg/stork.js";

// @TODO: Change this URL based on webpack production vs. development
// init("https://d1req3pu7uy8ci.cloudfront.net/stork.wasm");
init("http://localhost:8888/stork.wasm");

var entities = {};

const defaultConfig = {
  showProgress: true
};

var outputTemplate = `<div class="stork-message">{{message}}</div>
<ul class="stork-results">{{results}}</ul>`;

var htmlResultTemplate = `<li class="stork-result">
    <a href="{{link}}">
        <p class="stork-title">{{title}}</p>
        <p class="stork-excerpt">{{excerpt}}</p>
    </a>
</div>`;

function handleDownloadProgress(event, name) {
  let loadedPercentage = event.loaded / event.total;
  console.log(loadedPercentage);
  entities[name].progress = loadedPercentage;
  if (entities[name].config.showProgress) {
    updateDom(name);
  }
}

function handleLoadedIndex(event) {
  let response = event.target.response;
  let name = "federalist"; // @TODO Get actual name
  console.log(`${name}: ${response.byteLength} bytes loaded`);
  entities[name]["index"] = new Uint8Array(response);
  if (entities[name].elements.input.value) {
    performSearch(name, entities[name].elements.input.value);
  }
}

function loadIndexFromUrl(name, url, callbacks) {
  var r = new XMLHttpRequest();
  r.addEventListener("load", e => {
    if (callbacks.load) {
      callbacks.load(e);
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
  console.log(entities[name]);

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
  entities[name].elements.input.addEventListener("blur", e => {
    handleBlurEvent(e);
  });
}

async function resolveSearch(index, query) {
  if (!search) {
    return null;
  }
  return Array.from(JSON.parse(search(index, query)));
}

function handleInputEvent(event) {
  let name = event.target.getAttribute("data-stork");
  let query = event.target.value;

  if (!entities[name]["index"]) {
    return;
  }

  performSearch(name, query);
}

function handleBlurEvent(event) {
  let name = event.target.getAttribute("data-stork");
  if (event.target.value == "") {
    updateDom(name, "", "");
  }
}

function performSearch(name, query) {
  var message = "...";
  var resultString = "";

  if (query && query.length >= 3) {
    let results = resolveSearch(entities[name]["index"], query).then(
      results => {
        if (results && results.length === 0) {
          message = "No results found.";
        } else if (results && results.length > 0) {
          message = `${results.length} results found.`;
          for (let i = 0; i < results.length; i++) {
            let link = results[i]["entry"]["url"];
            let title = results[i]["entry"]["title"];
            let excerpt = results[i]["result"]["excerpts"].map(
              e => `${e.value}<br><br>`
            );
            let listItem = htmlResultTemplate
              .replace("{{link}}", link)
              .replace("{{title}}", title)
              .replace("{{excerpt}}", excerpt);
            resultString += listItem;
          }
        }
        updateDom(name, message, resultString);
      }
    );
  } else {
    updateDom(name, message, "");
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
    debugger;
    if (entity.elements.progress) {
      entity.elements.progress.style.width = `${entity.progress * 100}%`;
      entity.elements.progress.style.opacity = 0;
    }
  }

  // while (entities[name]["listElement"].firstChild) {
  //   entities[name]["listElement"].removeChild(
  //     entities[name]["listElement"].firstChild
  //   );
  // }
  // let messageElements = document.getElementsByClassName("stork-message");
  // for (let elem of messageElements) {
  //   elem.remove();
  // }
  // let messageString = messageTemplate.replace("{{message}}", message);
  // entities[name]["listElement"].insertAdjacentHTML(
  //   "beforebegin",
  //   messageString
  // );
  // entities[name]["listElement"].innerHTML = resultString;
}
