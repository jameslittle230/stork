const { search } = wasm_bindgen;
var stork = {
  entities: {},

  register: function(name, url) {
    stork.entities[name] = {};
    var r = new XMLHttpRequest();
    r.addEventListener("load", function(event) {
      let response = event.target.response;
      console.log(`${name}: ${response.byteLength} bytes loaded`);
      stork.entities[name]["index"] = new Uint8Array(response);
    });

    r.responseType = "arraybuffer";
    r.open("GET", "http://localhost:8000/out.stork");
    r.send();

    stork.entities[name]["listElement"] = document.querySelector(
      `ul[data-stork=${name}-results]`
    );

    stork.entities[name]["inputElement"] = document.querySelector(
      `input[data-stork=${name}]`
    );

    stork.entities[name]["inputElement"].addEventListener(
      "input",
      stork.performSearch
    );
  },

  performSearch: function(event) {
    let name = "federalist";
    let query = event.target.value;

    if (!stork.entities[name]["index"]) {
      return;
    }

    while (stork.entities[name]["listElement"].firstChild) {
      stork.entities[name]["listElement"].removeChild(
        stork.entities[name]["listElement"].firstChild
      );
    }

    if (query && query.length >= 3) {
      let results = Array.from(
        JSON.parse(search(stork.entities[name]["index"], query))
      );
      for (let i = 0; i < results.length; i++) {
        let li = document.createElement("li");
        li.appendChild(document.createTextNode(JSON.stringify(results[i])));
        stork.entities[name]["listElement"].appendChild(li);
      }
    }
  },

  init: async function() {
    await wasm_bindgen("./pkg/stork_bg.wasm");
  }
};

stork.init();
