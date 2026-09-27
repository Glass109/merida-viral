// Normal links and browser history remain in charge; this only styles their
// same-origin cross-document snapshots when both pages opted in via CSS.
(function () {
  const collectionPath = /^\/(venues|bands|artists|events)(?:\/(new|\d+))?\/?$/;

  function direction(activation) {
    if (!activation?.from || !activation.entry) return null;
    const from = new URL(activation.from.url);
    const to = new URL(activation.entry.url);
    if (from.origin !== to.origin || from.pathname === to.pathname) return null;
    const oldPage = collectionPath.exec(from.pathname === '/' ? '/events' : from.pathname);
    const newPage = collectionPath.exec(to.pathname === '/' ? '/events' : to.pathname);
    if (!oldPage || !newPage || oldPage[1] !== newPage[1]) return null;

    // Back/Forward traversals follow actual history, even between two details.
    if (activation.navigationType === "traverse" &&
        activation.from.index >= 0 && activation.entry.index >= 0) {
      return activation.entry.index < activation.from.index ? "metro-backward" : "metro-forward";
    }
    // A breadcrumb can navigate to the list by pushing a new history entry.
    const oldDepth = oldPage[2] ? 1 : 0;
    const newDepth = newPage[2] ? 1 : 0;
    if (oldDepth === newDepth) return null;
    return newDepth < oldDepth ? "metro-backward" : "metro-forward";
  }

  function prepare(event, activation) {
    if (!event.viewTransition) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      event.viewTransition.skipTransition();
      return;
    }
    const type = direction(activation);
    if (type) event.viewTransition.types.add(type);
    else event.viewTransition.skipTransition();
  }

  window.addEventListener("pageswap", (event) => prepare(event, event.activation));
  window.addEventListener("pagereveal", (event) => prepare(event, window.navigation?.activation));
})();
