# atsu-website-widgets

`atsu-website-widgets` turns your recent [atsu.moe](https://atsu.moe) reading into a row of cover cards for your GitHub profile or your own website.

Not affiliated with atsu.moe or AniList. The widget shows your recent reading, including titles and chapters, to anyone who can see the page it's on.

Reading from atsu.moe and the GitHub Action that refreshes the file are still being built. For now, `atsu-website-widgets draw --demo` writes the demo shelf.

## Embedding

The widget is a single SVG file, with its covers built in, so it works anywhere an image does.

### In a GitHub README

```markdown
![What I'm reading](shelf.svg)
```

An SVG inside an image can't see the page around it. The default grey text reads on both GitHub's light and dark themes, or you can fix the colours when drawing with `--heading-color`, `--title-color` and `--background`.

### On any website

```html
<img src="https://raw.githubusercontent.com/YOU/REPO/main/shelf.svg" alt="What I'm reading">
```

GitHub serves raw SVG files as images and allows other sites to fetch them, so this one URL works for both the plain image and the script below. GitHub caches raw files for five minutes, so a new shelf shows up shortly after it's committed.

### Matching your site's colours

Add `data-atsu-website-widgets` to the image, then load the script. Copy [`embed/atsu-website-widgets.js`](embed/atsu-website-widgets.js) into your site; it has no dependencies.

```html
<img data-atsu-website-widgets src="https://raw.githubusercontent.com/YOU/REPO/main/shelf.svg" alt="What I'm reading">
<script src="/atsu-website-widgets.js" defer></script>
```

The script fetches the SVG and places it directly in your page, so your CSS can colour it. Set these on the image's parent or any element above it:

```css
.reading {
  --atsu-website-widgets-heading: #1f2328;
  --atsu-website-widgets-title: #e75480;
  --atsu-website-widgets-background: transparent;
}
```

They're ordinary CSS custom properties, so they can follow your site's dark mode. [`embed/example.html`](embed/example.html) shows a page with a light and dark switch. To try it locally, run `python -m http.server` in the repo and open `http://localhost:8000/embed/example.html`, because browsers don't allow `fetch` on `file://` pages.

Some notes:

- If the script can't fetch or check the file, the plain image stays and a warning goes to the browser console.
- Before adding the SVG to your page, the script rebuilds it from a list of the elements the renderer produces. Scripts, event handlers, outside images, non-web links and styles that reach beyond the widget are all removed.
- If your site's Content Security Policy blocks inline styles, the widget's text loses its colours.
- On pages that add content with JavaScript, call `atsuShelf.load()` after inserting the image.
- The widget scales down to fit narrow screens. Six cards get small on a phone, so `--cards 3` or `4` may suit a mobile-first site.
