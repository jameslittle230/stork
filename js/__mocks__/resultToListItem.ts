import { ListItemDisplayOptions } from "../resultToListItem";
import { MockHtmlElement, createMockHtmlElement } from "./dom";

export function resultToListItem(
  options: ListItemDisplayOptions
): MockHtmlElement {
  return createMockHtmlElement();
}
