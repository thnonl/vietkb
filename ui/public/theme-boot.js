// Runs synchronously from <head>, before first paint. main.ts repeats this and
// the settings load overwrites it with the saved value.
try {
  var saved = localStorage.getItem("vkb:theme");
  document.documentElement.dataset.theme =
    saved === "light" || saved === "dark" ? saved : "system";
} catch (e) {
  document.documentElement.dataset.theme = "system";
}
