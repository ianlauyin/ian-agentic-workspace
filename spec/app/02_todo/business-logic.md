# TODO page

## Columns

- The page is a board with three columns, in this order: "Pending", "Ongoing" and "Reviewing".
- Each column has its own list of cards.
- The page has one Add button, at the right end of the title row, above the columns.
- If a column has no card, it shows only its drop section.

## Card

- Each TODO is a card. The card has a title, a detail and a deadline. The title is required. The detail and the deadline are optional.
- The Add button opens a modal form with a title input, a detail text area, an optional deadline date input, a Cancel button and an Add button. The title is required. Add or Enter in the title input or the deadline input adds the card to the end of the "Pending" column and closes the modal. Cancel, Escape or a click outside the modal closes it and adds nothing. The form is empty each time it opens.
- A card is a separate box with space between the cards. A card shows a top bar, its title, its detail if it has one, its deadline if it has one, an Edit button, and a Delete button.
- Edit changes the card to a form with a title input, a detail text area and a deadline date input. Save or Enter in an input keeps the change. Cancel or Escape discards it. The user clears the deadline date to remove the deadline.
- An empty title does not add or change a card.
- Delete asks for a confirmation before it removes the card.

## Move

- Each column ends with a drop section. It has a dashed border and the text "Drop a card here". It fills the rest of the height of the column. It shows the accent color while a card is over it.
- The user drags a card by its top bar to change its order, or to move it to another column. A line shows where the card goes. A drop on a column, but not on a card, puts the card at the end of that column and moves the card to that column. This includes a drop on the drop section. The drop saves the new lists.
- A moved card keeps its title, its detail and its deadline.

## Persist

- Each change saves the three lists with `PUT /page`, as `todo-items`, with the data `{"pending": [{"title": "...", "detail": "...", "deadline": "YYYY-MM-DD or empty"}], "ongoing": [...], "reviewing": [...]}`.
- When the page opens, it loads `todo-items` with `GET /page`.
- If the saved data has the old `important` list, the old `pending` list, or the old `items` list, the page loads these cards in the "Pending" column, with `important` first. An old card has `text`. The page loads `text` as the title, with an empty detail and an empty deadline. The next save writes the new data.
- If a save fails, the page shows the error and keeps the last saved lists.
- If the load fails, the page shows the error. The Add button stays disabled. Thus, a save cannot replace the saved lists with empty lists.
