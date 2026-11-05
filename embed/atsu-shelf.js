(() => {
  const SVG = "http://www.w3.org/2000/svg";
  const NUM = /^-?\d+(\.\d+)?$/;
  const SIZE = { width: NUM, height: NUM };
  const CLASS = /^[\w -]+$/;
  const ELEMENTS = {
    svg: { ...SIZE, viewBox: /^[\d. -]+$/, role: /^img$/ },
    title: {},
    style: {},
    g: { transform: /^translate\(-?[\d.]+[ ,]+-?[\d.]+\)$/ },
    a: { href: /^https?:\/\//i },
    clipPath: { id: /^[\w-]+$/ },
    rect: { ...SIZE, x: NUM, y: NUM, rx: NUM, fill: /^#[0-9a-f]{3,8}$/i },
    text: { x: NUM, y: NUM },
    image: {
      ...SIZE,
      preserveAspectRatio: /^[\w ]+$/,
      "clip-path": /^url\(#[\w-]+\)$/,
      href: /^data:image\/(png|jpeg);base64,[a-z0-9+/=]+$/i,
    },
  };
  const HAS_TEXT = new Set(["title", "text"]);
  const PROPERTIES = /^(--[\w-]+|fill|font-family|font-size|font-weight|opacity|text-anchor)$/;
  const pending = new WeakMap();
  let count = 0;

  function sanitize(text) {
    const doc = new DOMParser().parseFromString(text, "image/svg+xml");
    const root = doc.documentElement;
    if (doc.querySelector("parsererror") || root.namespaceURI !== SVG || root.localName !== "svg") {
      return null;
    }
    const prefix = `atsu-shelf-${++count}`;
    const svg = copy(root, prefix);
    svg?.classList.add(prefix);
    return svg ?? null;
  }

  function copy(node, prefix) {
    const allowed = ELEMENTS[node.localName];
    if (node.namespaceURI !== SVG || !allowed) return undefined;
    const el = document.createElementNS(SVG, node.localName);

    for (const { name, value, namespaceURI } of node.attributes) {
      const test = name === "class" ? CLASS : allowed[name];
      if (namespaceURI || !test?.test(value)) continue;
      const scoped =
        name === "id" ? `${prefix}-${value}` : name === "clip-path" ? value.replace("#", `#${prefix}-`) : value;
      el.setAttribute(name, scoped);
    }

    if (node.localName === "style") {
      const css = scope(node.textContent, prefix);
      if (css === null) return null;
      el.textContent = css;
      return el;
    }

    for (const child of node.childNodes) {
      if (child.nodeType === Node.TEXT_NODE) {
        if (HAS_TEXT.has(node.localName)) el.append(child.data);
      } else if (child.nodeType === Node.ELEMENT_NODE) {
        const copied = copy(child, prefix);
        if (copied === null) return null;
        if (copied) el.append(copied);
      }
    }
    return el;
  }

  function scope(css, prefix) {
    if (/[@\\<]|url\s*\(|image-set|src\s*\(|!important/i.test(css)) return null;
    const rules = [];
    for (const rule of css.split("}")) {
      if (!rule.trim()) continue;
      const match = /^\s*\.atsu-shelf((?:\s+\.[\w-]+)*)\s*\{([^{]*)$/.exec(rule);
      if (!match) return null;
      for (const decl of match[2].split(";")) {
        if (!decl.trim()) continue;
        const colon = decl.indexOf(":");
        if (colon < 0 || !PROPERTIES.test(decl.slice(0, colon).trim())) return null;
      }
      rules.push(`.${prefix}${match[1]}{${match[2]}}`);
    }
    return rules.join("\n");
  }

  async function inline(img) {
    try {
      const res = await fetch(img.currentSrc || img.src, { credentials: "omit" });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const svg = sanitize(await res.text());
      if (!svg) throw new Error("the file isn't an atsu-shelf SVG");
      if (img.alt) svg.setAttribute("aria-label", img.alt);
      svg.style.maxWidth = "100%";
      svg.style.height = "auto";
      img.replaceWith(svg);
    } catch (e) {
      console.warn(`atsu-shelf: kept the plain image for ${img.src}: ${e.message}`);
    }
  }

  function load(root = document) {
    const imgs = [...root.querySelectorAll("img[data-atsu-shelf]")];
    for (const img of imgs) if (!pending.has(img)) pending.set(img, inline(img));
    return Promise.all(imgs.map((img) => pending.get(img)));
  }

  window.atsuShelf = { load, sanitize };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => load());
  } else {
    load();
  }
})();
