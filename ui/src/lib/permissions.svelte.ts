// macOS permission state known before the first fit, so the window is sized once with
// the Accessibility banner in it (MAC-REVIEW: the window is staged hidden, so this
// resize is never seen; a late banner would make it jump by 80 px).
import * as api from "./api";
import { platformState } from "./platform.svelte";

export const perms = $state({ missing: false });

/** Asks the backend once. Browser preview and Windows skip it. */
export const probePermissions = async (): Promise<void> => {
  if (!api.isTauri() || !platformState.mac) return;
  try {
    const [ax, im] = await Promise.all([api.accessibilityStatus(), api.inputMonitoringStatus()]);
    perms.missing = !ax || !im;
  } catch {
    // Not answering: the banner's own poll decides later.
  }
};
