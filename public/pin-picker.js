(() => {
  const input = document.querySelector('#event-image');
  const pins = document.querySelectorAll('.pin-picker .photo-pin');
  if (!input) return;
  let preview;
  input.addEventListener('change', () => {
    if (preview) URL.revokeObjectURL(preview);
    const file = input.files[0];
    preview = file && ['image/jpeg', 'image/png', 'image/webp'].includes(file.type) ? URL.createObjectURL(file) : null;
    pins.forEach(pin => {
      const fill = pin.querySelector('.photo-pin-fill');
      const image = fill.querySelector('img');
      image.src = preview || '/noise.jpg';
      image.classList.toggle('photo-pin-placeholder', !preview);
      pin.querySelector('.photo-pin-rim img').src = preview || '/noise.jpg';
      fill.querySelector('.photo-pin-fallback')?.remove();
      if (!preview) {
        const fallback = document.createElement('span');
        fallback.className = 'photo-pin-fallback';
        fallback.textContent = '♪';
        fill.append(fallback);
      }
    });
  });
})();
