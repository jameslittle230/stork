import("../pkg/index.js")
  .catch(console.error)
  .then(module => {
    search = module.search;
  });

var entities = {};
export var search;

export function register(name, url) {
  entities[name] = {};
  var r = new XMLHttpRequest();
  r.addEventListener("load", function(event) {
    let response = event.target.response;
    console.log(`${name}: ${response.byteLength} bytes loaded`);
    entities[name]["index"] = new Uint8Array(response);
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
  entities[name]["inputElement"].addEventListener("input", performSearch);
}

async function resolveSearch(index, query) {
  // return new Promise(resolve => {
  return Array.from(JSON.parse(search(index, query)));
  // });
}

function performSearch(event) {
  let name = "federalist";
  let query = event.target.value;
  if (!entities[name]["index"]) {
    return;
  }
  while (entities[name]["listElement"].firstChild) {
    entities[name]["listElement"].removeChild(
      entities[name]["listElement"].firstChild
    );
  }
  if (query && query.length >= 3) {
    let results = resolveSearch(entities[name]["index"], query).then(
      results => {
        if (results.length === 0) {
          let text = document.createTextNode("No results found.");
          entities[name]["listElement"].appendChild(text);
          return;
        }

        for (let i = 0; i < results.length; i++) {
          let li = document.createElement("li");
          li.appendChild(document.createTextNode(JSON.stringify(results[i])));
          entities[name]["listElement"].appendChild(li);
        }
      }
    );
  } else {
    let text = document.createTextNode("...");
    entities[name]["listElement"].appendChild(text);
  }
}
