interface IndexLoaderCallbacks {
  load: (response: ArrayBufferLike) => void;
  progress: (percentage: number) => void;
  error: () => void;
}

export function loadIndexFromUrl(
  url: string,
  callbacks: IndexLoaderCallbacks
): void {
  const r = new XMLHttpRequest();

  r.addEventListener("load", e => {
    const { status, response } = e.target as XMLHttpRequest;

    // This shouldn't happen on the `load` event, but handle it safely if it does
    if (status === 0) {
      callbacks.progress(e.loaded / e.total);
      return;
    }

    if (status < 200 || status > 299) {
      callbacks.error();
      return;
    }

    callbacks.load(response);
  });

  r.addEventListener("error", () => {
    callbacks.error();
  });

  r.addEventListener("progress", e => {
    callbacks.progress(e.loaded / e.total);
  });

  r.responseType = "arraybuffer";
  r.open("GET", url);
  r.send();
}
