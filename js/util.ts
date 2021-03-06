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

export const plural = (
  count: number,
  singular: string,
  plural: string
): string => (count == 1 ? singular : plural);
