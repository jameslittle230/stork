// It's like Handlebars, but smaller.

export function highlight(text, highlight_ranges) {
  function insert(str, index, value) {
    return str.substr(0, index) + value + str.substr(index);
  }

  var charactersAlreadyAdded = 0;

  for (let range of highlight_ranges) {
    let beginningInsertion = `<span class="stork-highlight">`;
    let endInsertion = `</span>`;

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
