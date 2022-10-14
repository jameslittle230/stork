export interface MockHtmlElement {
  name: string;
  addEventListener: jest.MockedFunction<VoidFunction>;
  removeEventListener: jest.MockedFunction<VoidFunction>;
  insertAdjacentElement: jest.MockedFunction<VoidFunction>;
  remove: jest.MockedFunction<VoidFunction>;
  scrollIntoView: jest.MockedFunction<VoidFunction>;
  appendChild: jest.MockedFunction<VoidFunction>;
  classList: {
    entries: Array<string>;
    remove: jest.MockedFunction<VoidFunction>;
    add: jest.MockedFunction<VoidFunction>;
  };
  style: {
    width: string;
  };
  innerHTML: string;
  value: string;
  children: Array<MockHtmlElement>;
}

export const createMockHtmlElement = function (): MockHtmlElement {
  return {
    name: "",
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    insertAdjacentElement: jest.fn(),
    remove: jest.fn(),
    appendChild: jest.fn(),
    scrollIntoView: jest.fn(),
    classList: {
      entries: [],
      remove: jest.fn(),
      add: jest.fn()
    },
    style: {
      width: ""
    },
    innerHTML: "innerHTML",
    value: "value",
    children: []
  };
};

export const create = jest.fn(
  (name: string, attributes: Record<string, Array<string>>) => {
    const output = createMockHtmlElement();
    output.name = name;
    attributes.classNames.forEach(className => {
      output.classList.entries.push(className);
    });
    return output;
  }
);

export const add = jest.fn(
  (child: MockHtmlElement, _where: string, parent: MockHtmlElement) => {
    parent.children.push(child);
  }
);

export const clear = jest.fn((element: MockHtmlElement) => {
  element.children = [];
});

export const setText = jest.fn((element: MockHtmlElement, text: string) => {
  element.innerHTML = text;
});

export const existsBeyondContainerBounds = jest.fn(
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  (_elem: HTMLElement, _container: HTMLElement) => true
);
