import WasmLoader from "./wasmLoader";

export type EntityLoadValue = object;

export default class EntityLoader {
  wasmLoader: WasmLoader;

  constructor(wasmLoader: WasmLoader) {
    this.wasmLoader = wasmLoader;
  }

  load(url: string, progressCallback: (percentage: number) => void): Promise<ArrayBuffer> {
    return new Promise((resolve, reject) => {
      const request = new XMLHttpRequest();
      request.addEventListener("load", (event) => {
        const { status, response } = event.target as XMLHttpRequest;

        // This shouldn't happen on the `load` event, but handle it safely if it does
        if (status === 0) {
          progressCallback(event.loaded / event.total);
          return;
        }

        if (status < 200 || status > 299) {
          reject();
        } else {
          resolve(response);
        }
      });

      request.addEventListener("error", () => {
        reject();
      });

      request.addEventListener("progress", (e) => {
        progressCallback(e.loaded / e.total);
      });

      request.responseType = "arraybuffer";
      request.open("GET", url);
      request.send();
    });
  }
}
