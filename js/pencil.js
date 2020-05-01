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

export function generateListItem(r) {
  return `
<li class="stork-result">
    <a href="${r.entry.url}">
        <p class="stork-title">${highlight(
          r.entry.title,
          r.title_highlight_ranges
        )}</p>
        ${r.excerpts
          .map(
            e => `<p class="stork-excerpt">
          ...${highlight(e.text, e.highlight_ranges)}...
          </p>`
          )
          .join("")}
    </a>
</li>`;
}

export function generateScoresListItem(r) {
  console.log(highlight(r.entry.title, r.title_highlight_ranges));
  return `
<li class="stork-result">
    <a href="${r.entry.url}">
      <div style="display: flex; justify-content: space-between">
        <p class="stork-title">${highlight(
          r.entry.title,
          r.title_highlight_ranges
        )}</p>
        <code><b>${r.score}</b></code>
      </div>
        ${r.excerpts
          .map(
            e => `<div style="display: flex; justify-content: space-between">
              <p class="stork-excerpt">
                ...${highlight(e.text, e.highlight_ranges)}...
              </p>
              <code>${e.score}</code>
            </div>`
          )
          .join("")}
    </a>
</li>`;
}
