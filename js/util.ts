// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export function assert(condition: unknown, msg?: string): asserts condition {
  if (!condition) {
    throw new Error(msg);
  }
}

export function htmlToElement(html: string): ChildNode | null {
  const template = document.createElement("template");
  html = html.trim(); // Never return a text node of whitespace as the result
  template.innerHTML = html;
  return template.content.firstChild;
}

export function difference<T>(arr1: Array<T>, arr2: Array<T>): Array<T> {
  const set1 = new Set(arr1);
  const set2 = new Set(arr2);
  const diff = new Set(Array.from(set1).filter(x => !set2.has(x)));
  return Array.from(diff);
}
