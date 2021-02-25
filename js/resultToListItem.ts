import { highlight } from "./pencil";
import { Result } from "./searchData";

export interface ListItemDisplayOptions {
  selected: boolean;
  showScores: boolean;
}

export function resultToListItem(
  result: Result,
  options: ListItemDisplayOptions
): ChildNode {
  const template = document.createElement("template");
  template.innerHTML = `
<li class="stork-result${options.selected ? " selected" : ""}">
  <a href="${result.entry.url}">
    <div style="display: flex; justify-content: space-between">
      <p class="stork-title">${highlight(
        result.entry.title,
        result.title_highlight_ranges || []
      )}</p>
      ${options.showScores ? `<code><b>${result.score}</b></code>` : ""}
    </div>
      ${result.excerpts
        .map(
          e => `<div style="display: flex; justify-content: space-between"><p class="stork-excerpt">
        ...${highlight(e.text, e.highlight_ranges || [])}...
        </p>
        ${options.showScores ? `<code>${e.score}</code>` : ""}
        </div>`
        )
        .join("")}
  </a>
</li>`;
  return template.content.firstElementChild as ChildNode;
}
