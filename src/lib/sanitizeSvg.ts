const BLOCKED_ELEMENTS = new Set([
  "script",
  "foreignobject",
  "iframe",
  "object",
  "embed",
  "link",
  "meta",
  "style",
  "image",
  "audio",
  "video",
]);

const URL_ATTRIBUTES = new Set(["href", "xlink:href", "src"]);

export function sanitizeSvg(svg: string): string | null {
  if (typeof window === "undefined" || !svg.trim()) return null;

  const doc = new DOMParser().parseFromString(svg, "image/svg+xml");
  if (doc.querySelector("parsererror")) return null;

  const root = doc.documentElement;
  if (root.tagName.toLowerCase() !== "svg") return null;

  for (const element of Array.from(root.querySelectorAll("*"))) {
    const tagName = element.tagName.toLowerCase();
    if (BLOCKED_ELEMENTS.has(tagName)) {
      element.remove();
      continue;
    }

    for (const attr of Array.from(element.attributes)) {
      const name = attr.name.toLowerCase();
      const value = attr.value.trim().toLowerCase();

      if (
        name.startsWith("on") ||
        name === "style" ||
        URL_ATTRIBUTES.has(name) ||
        value.includes("javascript:") ||
        value.includes("data:text/html")
      ) {
        element.removeAttribute(attr.name);
      }
    }
  }

  for (const attr of Array.from(root.attributes)) {
    const name = attr.name.toLowerCase();
    const value = attr.value.trim().toLowerCase();
    if (name.startsWith("on") || name === "style" || value.includes("javascript:")) {
      root.removeAttribute(attr.name);
    }
  }

  return new XMLSerializer().serializeToString(root);
}
