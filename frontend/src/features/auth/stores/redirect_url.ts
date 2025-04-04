export const REDIRECT_KEY = "redirect_url";

/**
 * Change the current path to the url stored as `REDIRECT_KEY`.
 * This just change history, so is needed to listen history to
 * react, also prevent screen flickering.
 */
export function goToRedirectUrl() {
  const redirect_url = window.localStorage.getItem(REDIRECT_KEY) ?? "/";
  window.localStorage.removeItem(REDIRECT_KEY);

  const url = new URL(window.location.href);
  url.search = "";
  url.pathname = redirect_url;

  window.history.replaceState({}, "", url);
}

/**
 * Send actual pathname to localStorage as `REDIRECT_KEY`.
 */
export function saveRedirectUrl() {
  window.localStorage.setItem(REDIRECT_KEY, window.location.pathname);
}
