export default class Dom {
  create(name, attributes) {
    let elem = document.createElement(name);
    if (attributes.classNames) {
      elem.setAttribute("class", attributes.classNames.join(" "));
    }
    return elem;
  }

  add(elem, location, reference) {
    reference.insertAdjacentElement(location, elem);
  }

  clear(elem) {
    while (elem.firstChild) {
      elem.removeChild(elem.firstChild);
    }
  }

  setText(elem, text) {
    let textNode = document.createTextNode(text);
    if (elem.firstChild) {
      elem.replaceChild(textNode, elem.firstChild);
    } else {
      elem.appendChild(textNode);
    }
  }
}
