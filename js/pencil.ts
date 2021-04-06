// It's like Handlebars, but smaller.

import { HighlightRange } from "./searchData";

export function highlight(
  text: string,
  highlight_ranges: Array<HighlightRange>
): string {
  function insert(str: string, index: number, value: string) {
    return str.substr(0, index) + value + str.substr(index);
  }

  let charactersAlreadyAdded = 0;

  for (const range of highlight_ranges) {
    const beginningInsertion = `<mark class="stork-highlight">`;
    const endInsertion = `</mark>`;

    text = insert(
      text,
      range.beginning + charactersAlreadyAdded,
      beginningInsertion
    );
    charactersAlreadyAdded += beginningInsertion.length;

    text = insert(text, range.end + charactersAlreadyAdded, endInsertion);
    charactersAlreadyAdded += endInsertion.length;
  }

  return text;
}
