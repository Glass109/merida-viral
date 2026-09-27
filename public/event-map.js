(() => {
  const map = mvMap.createMap(document.querySelector('#event-map'), mvMap.CENTER, 12);
  const rows = [...document.querySelectorAll('.event-discovery-list > li')];
  const bounds = new maplibregl.LngLatBounds();
  const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
  // Co-located shows remain individually discoverable through the list.
  const places = new Map();
  rows.forEach(row => {
    const point = mvMap.coordinatesFromElement(row);
    if (!point) return;
    bounds.extend(point);
    const key = point.join(',');
    if (!places.has(key)) places.set(key, []);
    places.get(key).push(row);
  });
  places.forEach(group => {
    const pin = document.createElement('button');
    pin.type = 'button';
    pin.className = 'event-map-pin';
    pin.setAttribute('aria-label', group.map(row => row.dataset.title).join(' · '));
    const marker = mvMap.addPlacePin(map, group[0], pin, { anchor: 'bottom' });
    let selected = 0;
    pin.addEventListener('click', () => {
      const row = group[selected++ % group.length];
      row.scrollIntoView({ behavior: reduced() ? 'instant' : 'smooth', block: 'nearest' });
      row.querySelector('a').focus({ preventScroll: true });
    });
    group.forEach(row => row.querySelector('.event-locate').addEventListener('click', () => {
      selected = group.indexOf(row);
      pin.replaceChildren(row.querySelector('template').content.cloneNode(true));
      map.flyTo({ center: marker.getLngLat(), zoom: 15, duration: reduced() ? 0 : 450 });
      document.querySelector('#event-map').scrollIntoView({ behavior: reduced() ? 'instant' : 'smooth', block: 'center' });
      pin.focus({ preventScroll: true });
    }));
  });
  if (!bounds.isEmpty()) map.fitBounds(bounds, { padding: 75, maxZoom: 14, duration: 0 });
})();
