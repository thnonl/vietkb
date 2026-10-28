<script lang="ts">
  // Inline SVG icon set (contract §7: no icon package). 20 × 20 box, 1.5 px
  // stroke; `small` uses the 16 px placement rule (stroke 1.875 from kit.css).
  type Name =
    | "search"
    | "plus"
    | "minus"
    | "check"
    | "close"
    | "chevron"
    | "arrow-right"
    | "edit"
    | "more"
    | "trash"
    | "reset"
    | "folder"
    | "download"
    | "upload"
    | "warning"
    | "info"
    | "shield"
    | "copy"
    | "sliders"
    | "apps";

  const ICONS: Record<Name, string> = {
    search: '<circle cx="9" cy="9" r="5.25"/><path d="M13 13l3.25 3.25"/>',
    plus: '<path d="M10 4.75v10.5M4.75 10h10.5"/>',
    minus: '<path d="M4.75 10h10.5"/>',
    check: '<path d="M4.75 10.25l3.5 3.5 7-7"/>',
    close: '<path d="M5.75 5.75l8.5 8.5M14.25 5.75l-8.5 8.5"/>',
    chevron: '<path d="M6.5 8.25 10 11.75l3.5-3.5"/>',
    "arrow-right": '<path d="M8.25 6.5 11.75 10l-3.5 3.5"/>',
    edit:
      '<path d="M12.6 4.4a1.6 1.6 0 0 1 2.3 0l.7.7a1.6 1.6 0 0 1 0 2.3l-8.1 8.1-3.4.75.75-3.4z"/>',
    more:
      '<circle cx="5.5" cy="10" r=".9" fill="currentColor" stroke="none"/><circle cx="10" cy="10" r=".9" fill="currentColor" stroke="none"/><circle cx="14.5" cy="10" r=".9" fill="currentColor" stroke="none"/>',
    trash:
      '<path d="M4.25 6h11.5M8 6V4.5h4V6M5.75 6l.7 9.75h7.1l.7-9.75"/>',
    reset: '<path d="M4.6 8.75a5.6 5.6 0 1 1 .95 4.4"/><path d="M4.25 4.75v4h4"/>',
    folder:
      '<path d="M3.25 6v8.5c0 .69.56 1.25 1.25 1.25h11c.69 0 1.25-.56 1.25-1.25v-6.5c0-.69-.56-1.25-1.25-1.25H10.1L8.4 4.75H4.5c-.69 0-1.25.56-1.25 1.25z"/>',
    download:
      '<path d="M10 3.75v8.5M6.75 9.25 10 12.5l3.25-3.25"/><path d="M4 13.25v1.5c0 .83.67 1.5 1.5 1.5h9c.83 0 1.5-.67 1.5-1.5v-1.5"/>',
    upload:
      '<path d="M10 12.25v-8.5M6.75 7 10 3.75 13.25 7"/><path d="M4 13.25v1.5c0 .83.67 1.5 1.5 1.5h9c.83 0 1.5-.67 1.5-1.5v-1.5"/>',
    warning:
      '<path d="M8.7 4.25a1.5 1.5 0 0 1 2.6 0l5.3 9.4a1.5 1.5 0 0 1-1.3 2.25H4.7a1.5 1.5 0 0 1-1.3-2.25z"/><path d="M10 8.25v3"/><circle cx="10" cy="13.55" r=".8" fill="currentColor" stroke="none"/>',
    info: '<circle cx="10" cy="10" r="6.25"/><path d="M10 9v4"/><circle cx="10" cy="6.9" r=".8" fill="currentColor" stroke="none"/>',
    shield:
      '<path d="M10 3.25 15.5 5.25v4.5c0 3.1-2.2 5.45-5.5 7-3.3-1.55-5.5-3.9-5.5-7v-4.5z"/>',
    copy: '<rect x="7.5" y="7.5" width="8.5" height="8.5" rx="1.75"/><path d="M12.25 4.5H5.5A1.5 1.5 0 0 0 4 6v6.75"/>',
    sliders:
      '<path d="M3.25 6.5h7M13.5 6.5h3.25M3.25 13.5h3.5M10 13.5h6.75"/><circle cx="12" cy="6.5" r="1.65"/><circle cx="8.5" cy="13.5" r="1.65"/>',
    apps:
      '<rect x="3.5" y="3.5" width="5.5" height="5.5" rx="1.4"/><rect x="11" y="3.5" width="5.5" height="5.5" rx="1.4"/><rect x="3.5" y="11" width="5.5" height="5.5" rx="1.4"/><rect x="11" y="11" width="5.5" height="5.5" rx="1.4"/>',
  };

  let { name, small = false }: { name: Name; small?: boolean } = $props();
</script>

<svg class="i" class:s16={small} viewBox="0 0 20 20" aria-hidden="true" focusable="false">
  {@html ICONS[name] ?? ""}
</svg>
