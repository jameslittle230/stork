import { Entity } from "./entity";
import { Configuration, calculateOverriddenConfig } from "./config";
import { loadIndexFromUrl } from "./loaders/indexLoader";
import { runAfterWasmLoaded } from "./wasmManager";
import StorkError from "./storkError";

const entities: Record<string, Entity> = {};

const register = (
  name: string,
  url: string,
  partialConfig: Partial<Configuration>
) => {
  return new Promise((res, rej) => {
    const fullConfig = calculateOverriddenConfig(partialConfig);
    if (fullConfig instanceof StorkError) {
      rej(fullConfig);
      return;
    }

    if (entities[name] && !fullConfig.forceOverwrite) {
      throw new StorkError(
        `You're registering an index named \`${name}\`, but that already exists. If this is expected, set forceOverwrite to true in your Javascript config to allow overwriting indexes.`
      );
    }

    const entity = new Entity(name, url, fullConfig);
    entities[name] = entity;

    loadIndexFromUrl(url, {
      progress: percentage => {
        entity.setDownloadProgress(percentage);
      },

      load: response => {
        runAfterWasmLoaded(() => {
          entity.registerIndex(new Uint8Array(response)).then(res).catch(rej);
        });
      },

      error: () => {
        entity.setDownloadError();
        rej();
      }
    });
  });
};

const attachToDom = (name: string) => {
  if (!entities[name]) {
    throw new Error(`Index ${name} has not been registered!`);
  }

  entities[name].attachToDom();
};

const entityIsReady = (name: string) => {
  return entities[name]?.state != "ready"
}

const debug = () => ({
  entities: {...entities},
  entitiesCount: entities.length
})

export { register, attachToDom, entityIsReady, debug };
