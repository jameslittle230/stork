export default class Dom {
  static create(name, attributes) {
    const elem = document.createElement(name);
    if (attributes.classNames) {
      elem.setAttribute("class", attributes.classNames.join(" "));
    }
    return elem;
  }

  static add(elem, location, reference) {
    reference.insertAdjacentElement(location, elem);
  }

  static clear(elem) {
    while (elem.firstChild) {
      elem.removeChild(elem.firstChild);
    }
  }

  static setText(elem, text) {
    const textNode = document.createTextNode(text);
    if (elem.firstChild) {
      elem.replaceChild(textNode, elem.firstChild);
    } else {
      elem.appendChild(textNode);
    }
  }
}
