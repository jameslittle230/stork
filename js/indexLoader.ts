import { Entity } from "./entity";

interface IndexLoaderCallbacks {
  load: (event: Event, entity: Entity) => void;
  progress: (percentage: number, entity: Entity) => void;
  error: () => void;
}

const FAKE_PROGRESS_BUMP_ON_START = 0.03;
const PROGRESS_SHOWING_INDEX_NOT_YET_REGISTERED = 0.94;

export function loadIndexFromUrl(
  entity: Entity,
  url: string,
  callbacks: IndexLoaderCallbacks
): void {
  const r = new XMLHttpRequest();

  r.addEventListener("load", e => {
    // This gets called even if we get a 404 response from the server!
    if (callbacks.load) {
      callbacks.load(e, entity);
    }
  });

  r.addEventListener("error", () => {
    console.error(`Could not fetch ${url}`);
    callbacks.error();
  });

  r.addEventListener("progress", e => {
    if (callbacks.progress) {
      const loadedPercentage = Math.min(
        Math.max(FAKE_PROGRESS_BUMP_ON_START, e.loaded / e.total),
        PROGRESS_SHOWING_INDEX_NOT_YET_REGISTERED
      );
      callbacks.progress(loadedPercentage, entity);
    }
  });

  if (callbacks.progress) {
    callbacks.progress(FAKE_PROGRESS_BUMP_ON_START, entity);
  }

  r.responseType = "arraybuffer";
  r.open("GET", url);
  r.send();
}
