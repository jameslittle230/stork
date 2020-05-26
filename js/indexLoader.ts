import { Entity } from "./entity";

interface IndexLoaderCallbacks {
  load: (event: Event, entity: Entity) => void;
  progress: (percentage: number, entity: Entity) => void;
}

const FAKE_PROGRESS_BUMP_ON_START = 0.03;

export function loadIndexFromUrl(
  entity: Entity,
  url: string,
  callbacks: IndexLoaderCallbacks
): void {
  const r = new XMLHttpRequest();

  r.addEventListener("load", e => {
    if (callbacks.load) {
      callbacks.load(e, entity);
    }
  });

  r.addEventListener("progress", e => {
    if (callbacks.progress) {
      const loadedPercentage = Math.max(
        FAKE_PROGRESS_BUMP_ON_START,
        e.loaded / e.total
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
