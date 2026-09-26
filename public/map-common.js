(function () {
  const STYLES = {
    light: "https://tiles.openfreemap.org/styles/positron",
    dark: "https://tiles.openfreemap.org/styles/dark",
  };

  function currentTheme() {
    return document.documentElement.dataset.theme === "light" ? "light" : "dark";
  }

  function styleFor(theme) {
    return STYLES[theme || currentTheme()] || STYLES.dark;
  }

  // OpenMapTiles ships named landmarks in the `poi` source-layer (shops,
  // including malls, plus hospitals, museums, markets…). Positron and Dark
  // don't render that layer, so we draw text-only labels, tiered by importance.
  // Transit/utility POIs are skipped so real destinations aren't drowned out.
  const NOISY_CLASSES = [
    "bus",
    "railway",
    "aerialway",
    "parking",
    "toilets",
    "drinking_water",
    "waste_basket",
    "atm",
    "bicycle",
    "car",
    "entrance",
    "post",
    "information",
  ];

  function landmarkLayers(theme) {
    const dark = (theme || currentTheme()) === "dark";
    const text = dark ? "#cfcfcf" : "#4a4a4a";
    const halo = dark ? "#0c0c0c" : "#ffffff";
    const nameField = ["coalesce", ["get", "name:latin"], ["get", "name"], ["get", "name_en"]];
    const base = {
      type: "symbol",
      source: "openmaptiles",
      "source-layer": "poi",
      layout: {
        "text-field": nameField,
        "text-font": ["Noto Sans Regular"],
        "text-size": 12,
        "text-max-width": 9,
        "text-anchor": "top",
        "text-offset": [0, 0.5],
        "symbol-sort-key": ["get", "rank"],
      },
      paint: {
        "text-color": text,
        "text-halo-color": halo,
        "text-halo-width": 1,
        "text-halo-blur": 0.5,
      },
    };
    const notNoisy = ["!", ["in", ["get", "class"], ["literal", NOISY_CLASSES]]];
    return [
      // Malls are the most common video backdrop, so give them their own
      // higher-contrast layer that wins label collisions.
      Object.assign({}, base, {
        id: "mv-poi-mall",
        minzoom: 14,
        filter: ["all", ["has", "name"], ["==", ["get", "subclass"], "mall"]],
        layout: Object.assign({}, base.layout, {
          "text-font": ["Noto Sans Bold"],
          "text-size": 13,
          "text-allow-overlap": true,
          "symbol-sort-key": 0,
        }),
      }),
      Object.assign({}, base, {
        id: "mv-poi-major",
        minzoom: 14,
        filter: ["all", ["has", "rank"], ["<", ["get", "rank"], 7], notNoisy],
      }),
      Object.assign({}, base, {
        id: "mv-poi-minor",
        minzoom: 16,
        filter: ["all", ["has", "rank"], [">=", ["get", "rank"], 7], notNoisy],
      }),
    ];
  }

  function addLandmarks(map) {
    for (const layer of landmarkLayers()) {
      if (!map.getLayer(layer.id)) map.addLayer(layer);
    }
  }

  // Wire a map to theme changes and keep landmark labels across style swaps.
  function setup(map) {
    map.on("style.load", () => addLandmarks(map));
    document.addEventListener("mv:themechange", (event) => {
      // diff: false forces a full style reload so `style.load` fires and the
      // landmark layers are rebuilt with the new theme's colors.
      map.setStyle(styleFor(event.detail.theme), { diff: false });
    });
  }

  window.mvMap = { styleFor, setup };
})();
