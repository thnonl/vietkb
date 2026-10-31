// Runs synchronously from <head>, before first paint. The shell injects the
// saved theme as window.__VKB_THEME__ (window_bg.rs); localStorage is the
// fallback, then the OS. main.ts repeats this and the settings load
// overwrites it with the saved value.
try {
  var injected = window.__VKB_THEME__;
  var saved = null;
  if (injected !== "light" && injected !== "dark" && injected !== "system") {
    saved = localStorage.getItem("vkb:theme");
  }
  document.documentElement.dataset.theme =
    injected === "light" || injected === "dark" || injected === "system"
      ? injected
      : saved === "light" || saved === "dark"
        ? saved
        : "system";
} catch (e) {
  document.documentElement.dataset.theme = "system";
}
