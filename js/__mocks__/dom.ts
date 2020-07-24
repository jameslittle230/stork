export const mockHtmlElement = {
  addEventListener: jest.fn(),
  insertAdjacentElement: jest.fn(),
  remove: jest.fn(),
  appendChild: jest.fn(),
  classList: {
    remove: jest.fn(),
    add: jest.fn()
  },
  style: {
    width: ""
  },
  innerHTML: "innerhtml",
  value: "value"
};

export const create = jest.fn(
  (name: string, attributes: Record<string, Array<string>>) => mockHtmlElement
);

export const add = jest.fn();
export const clear = jest.fn();
export const setText = jest.fn();
export const existsBeyondContainerBounds = jest.fn(
  (elem: HTMLElement, container: HTMLElement) => true
);
