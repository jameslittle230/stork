// It's like Handlebars, but smaller.

function highlight(text, highlight_ranges) {
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

export function generateListItem(options) {
  return `
<li class="stork-result${options.selected ? " selected" : ""}">
  <a href="${options.result.entry.url}">
    <div style="display: flex; justify-content: space-between">
      <p class="stork-title">${highlight(
        options.result.entry.title,
        options.result.title_highlight_ranges
      )}</p>
      ${options.showScores ? `<code><b>${options.result.score}</b></code>` : ""}
    </div>
      ${options.result.excerpts
        .map(
          e => `<div style="display: flex; justify-content: space-between"><p class="stork-excerpt">
        ...${highlight(e.text, e.highlight_ranges)}...
        </p>
        ${options.showScores ? `<code>${e.score}</code>` : ""}
        </div>`
        )
        .join("")}
  </a>
</li>`;
}
