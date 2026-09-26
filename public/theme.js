(function () {
  const root = document.documentElement;
  const media = window.matchMedia("(prefers-color-scheme: light)");
  const THEMES = ["system", "dark", "light"];
  const ACCENTS = ["blue", "plum", "green"];

  const savedTheme = localStorage.getItem("mv-theme");
  const savedAccent = localStorage.getItem("mv-accent");
  let themePreference = THEMES.includes(savedTheme) ? savedTheme : "system";
  let accent = ACCENTS.includes(savedAccent) ? savedAccent : "blue";

  const settingsButton = document.querySelector("#settings-toggle");
  const settingsPanel = document.querySelector("#settings-panel");
  const themeColor = document.querySelector('meta[name="theme-color"]');

  function systemTheme() {
    return media.matches ? "light" : "dark";
  }
  function resolvedTheme() {
    return themePreference === "system" ? systemTheme() : themePreference;
  }
  function syncControls() {
    if (!settingsPanel) return;
    const themeRadio = settingsPanel.querySelector(`input[name="theme"][value="${themePreference}"]`);
    if (themeRadio) themeRadio.checked = true;
    const accentRadio = settingsPanel.querySelector(`input[name="accent"][value="${accent}"]`);
    if (accentRadio) accentRadio.checked = true;
  }
  function applyTheme() {
    root.dataset.theme = resolvedTheme();
    if (themeColor) themeColor.content = root.dataset.theme === "light" ? "#ffffff" : "#101010";
    document.dispatchEvent(new CustomEvent("mv:themechange", { detail: { theme: root.dataset.theme } }));
  }

  root.dataset.theme = resolvedTheme();
  root.dataset.accent = accent;
  if (themeColor) themeColor.content = root.dataset.theme === "light" ? "#ffffff" : "#101010";

  if (settingsButton && settingsPanel) {
    syncControls();

    function closeSettings() {
      settingsPanel.hidden = true;
      settingsButton.setAttribute("aria-expanded", "false");
    }
    settingsButton.addEventListener("click", () => {
      settingsPanel.hidden = !settingsPanel.hidden;
      settingsButton.setAttribute("aria-expanded", String(!settingsPanel.hidden));
    });
    settingsPanel.addEventListener("change", (event) => {
      if (event.target.name === "theme") {
        themePreference = THEMES.includes(event.target.value) ? event.target.value : "system";
        localStorage.setItem("mv-theme", themePreference);
        applyTheme();
      } else if (event.target.name === "accent") {
        accent = event.target.value;
        localStorage.setItem("mv-accent", accent);
        root.dataset.accent = accent;
      }
    });
    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && !settingsPanel.hidden) {
        closeSettings();
        settingsButton.focus();
      }
    });
    document.addEventListener("click", (event) => {
      if (!settingsPanel.hidden && !settingsPanel.contains(event.target) && !settingsButton.contains(event.target)) closeSettings();
    });
  }

  // Follow the OS only while the user hasn't pinned an explicit theme.
  media.addEventListener("change", () => {
    if (themePreference === "system") applyTheme();
  });
})();
