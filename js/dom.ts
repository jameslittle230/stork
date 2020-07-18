export function create(
  name: string,
  attributes: Record<string, Array<string>>
): HTMLElement {
  const elem = document.createElement(name);
  if (attributes.classNames) {
    elem.setAttribute("class", attributes.classNames.join(" "));
  }
  return elem;
}

export function add(
  elem: HTMLElement,
  location: InsertPosition,
  reference: HTMLElement
): void {
  reference.insertAdjacentElement(location, elem);
}

export function clear(elem: HTMLElement | null): void {
  while (elem && elem.firstChild) {
    elem.removeChild(elem.firstChild);
  }
}

export function setText(elem: HTMLElement | null, text: string): void {
  const textNode = document.createTextNode(text);
  if (elem && elem.firstChild) {
    elem.replaceChild(textNode, elem.firstChild);
  } else if (elem) {
    elem.appendChild(textNode);
  }
}

export function existsBeyondContainerBounds(
  elem: HTMLElement,
  container: HTMLElement
): boolean {
  const elemBoundingBox = elem.getBoundingClientRect();
  const containerBoundingBox = container.getBoundingClientRect();

  return (
    elemBoundingBox.bottom > containerBoundingBox.bottom ||
    elemBoundingBox.top < containerBoundingBox.top
  );
}
