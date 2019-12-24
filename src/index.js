import("../pkg/index.js")
  .catch(console.error)
  .then(module => {
    search = module.search;
    for (name in entities) {
      if (entities[name]["inputElement"].value) {
        performSearch(name, entities[name]["inputElement"].value);
      }
    }
  });

var entities = {};
export var search;

const messageTemplate = `<div class="stork-message">{{message}}</div>`;

const htmlResultTemplate = `<li class="stork-result">
    <a href="{{link}}">
        <p class="stork-title">{{title}}</p>
        <p class="stork-excerpt">{{excerpt}}</p>
    </a>
</div>`;

export function register(name, url) {
  entities[name] = {};
  var r = new XMLHttpRequest();
  r.addEventListener("load", e => {
    handleLoadedIndex(name, e);
  });
  r.responseType = "arraybuffer";
  r.open("GET", "http://localhost:8000/out.stork");
  r.send();
  entities[name]["listElement"] = document.querySelector(
    `ul[data-stork=${name}-results]`
  );
  entities[name]["inputElement"] = document.querySelector(
    `input[data-stork=${name}]`
  );
  entities[name]["inputElement"].addEventListener("input", handleInputEvent);
  entities[name]["inputElement"].addEventListener("blur", e => {
    handleBlurEvent(e);
  });
}

function handleLoadedIndex(name, event) {
  let response = event.target.response;
  console.log(`${name}: ${response.byteLength} bytes loaded`);
  entities[name]["index"] = new Uint8Array(response);
  if (entities[name]["inputElement"].value) {
    performSearch(name, entities[name]["inputElement"].value);
  }
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
  updateDom(name, "", "");
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
            let link = results[i]["url"];
            let title = results[i]["title"];
            let excerpt = results[i]["excerpt"];
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

function updateDom(name, message, resultString) {
  while (entities[name]["listElement"].firstChild) {
    entities[name]["listElement"].removeChild(
      entities[name]["listElement"].firstChild
    );
  }

  let messageElements = document.getElementsByClassName("stork-message");
  for (let elem of messageElements) {
    elem.remove();
  }

  let messageString = messageTemplate.replace("{{message}}", message);
  entities[name]["listElement"].insertAdjacentHTML(
    "beforebegin",
    messageString
  );
  entities[name]["listElement"].innerHTML = resultString;
}
