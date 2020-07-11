/* eslint-disable no-undef */

import { assert, difference } from "./util";
import { defaultConfig } from "./config";
import { loadWasm } from "./wasmLoader";
import { EntityManager } from "./entityManager";

const wasmQueue = loadWasm();
const entityManager = new EntityManager(wasmQueue);

export function register(name, url, config = {}) {
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

export default {
  register
};
