/* eslint-disable no-undef */

import { difference } from "./util";
import { defaultConfig } from "./config";
import { createWasmQueue } from "./wasmLoader";
import { EntityManager } from "./entityManager";

var wasmQueue = null;
const entityManager = new EntityManager();

export function initialize() {
  if (!wasmQueue) {
    wasmQueue = createWasmQueue();
    entityManager.wasmQueue = wasmQueue;
  }
}

export function register(name, url, config = {}) {
  initialize();
  if (typeof name !== "string") {
    throw new Error("Index registration name must be a string.");
  }

  if (typeof url !== "string") {
    throw new Error("URL must be a string.");
  }

  let configKeyDiff = difference(
    Object.keys(config),
    Object.keys(defaultConfig)
  );
  if (configKeyDiff.size > 0) {
    throw new Error(
      `Invalid key${
        configKeyDiff > 1 ? "s" : ""
      } in config object: ${JSON.stringify(Array.from(configKeyDiff))}`
    );
  }

  entityManager.register(name, url, config);
}

export function downloadIndex(name, url, config = {}) {
  if (typeof name !== "string") {
    throw new Error("Index registration name must be a string.");
  }

  if (typeof url !== "string") {
    throw new Error("URL must be a string.");
  }

  if (!wasmQueue) {
    throw new Error(
      "Run stork.initialize() before running stork.donwloadIndex()"
    );
  }

  let configKeyDiff = difference(
    Object.keys(config),
    Object.keys(defaultConfig)
  );

  if (configKeyDiff.size > 0) {
    throw new Error(
      `Invalid key${
        configKeyDiff > 1 ? "s" : ""
      } in config object: ${JSON.stringify(Array.from(configKeyDiff))}`
    );
  }

  entityManager.register(name, url, config);
}

export function attach(name) {
  console.log(entityManager.entities.map(o => o.name).includes(name));
  if (!entityManager.entities.keys.includes(name)) {
    throw new Error(
      `No index ${name} - Make sure to call stork.downloadIndex() before calling stork.attach().`
    );
  }
  entityManager.attachToDom(name);
}

export default {
  register,
  initialize,
  downloadIndex,
  attach
};
