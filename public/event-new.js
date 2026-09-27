const eventForm = document.querySelector("#event-form");
const bandChoices = [...eventForm.querySelectorAll('input[name="band_id"]')];

for (const checkbox of bandChoices) {
  checkbox.addEventListener("change", () => bandChoices[0].setCustomValidity(""));
}
eventForm.addEventListener("submit", (event) => {
  if (bandChoices.some((checkbox) => checkbox.checked)) return;
  event.preventDefault();
  bandChoices[0].setCustomValidity(eventForm.dataset.bandRequired);
  bandChoices[0].reportValidity();
});
