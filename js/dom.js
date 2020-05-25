declare var document;

export function create(name, attributes) {
  const elem = document.createElement(name);
  if (attributes.classNames) {
    elem.setAttribute("class", attributes.classNames.join(" "));
  }
  return elem;
}

export function add(elem, location, reference) {
  reference.insertAdjacentElement(location, elem);
}

export function clear(elem) {
  while (elem.firstChild) {
    elem.removeChild(elem.firstChild);
  }
}

export function setText(elem, text) {
  const textNode = document.createTextNode(text);
  if (elem.firstChild) {
    elem.replaceChild(textNode, elem.firstChild);
  } else {
    elem.appendChild(textNode);
  }
}
