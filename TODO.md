# TODO

- [ ] Scrollback / history navigation in the ratatui UI.
  Currently the message pane auto-sticks to the bottom. Add PgUp/PgDn
  (and optionally arrow keys when the input line is empty) to scroll the
  message buffer. Track a scroll offset in `AppState`, detach from
  stick-to-bottom when the user scrolls up, and re-attach when they
  return to the latest line. Render only the visible window.
