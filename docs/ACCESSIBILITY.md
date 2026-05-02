# Accessibility Guidelines for ChainLogistics

This document provides accessibility guidelines for the ChainLogistics frontend to ensure WCAG 2.1 AA compliance and provide an excellent experience for all users.

## Overview

Accessibility is a core requirement for ChainLogistics. We strive for:
- **WCAG 2.1 Level AA** compliance
- **Screen reader** compatibility
- **Keyboard navigation** support
- **Sufficient color contrast** (at least 4.5:1 for text)
- **Mobile accessibility**

## Keyboard Navigation

All interactive elements must be keyboard accessible:

### Focus Management
```tsx
// Good: Elements are naturally focusable or have tabIndex={0}
<button onClick={handleClick}>Delete</button>
<a href="/page">Link</a>

// Avoid: Non-interactive elements without keyboard support
<div onClick={handleClick}>Delete</div> // ❌

// Better: Make it interactive
<div role="button" tabIndex={0} onClick={handleClick} onKeyDown={handleEnter}>
  Delete
</div>
```

### Focus Indicators
- **Always maintain visible focus indicators** - Do not remove outline/focus-visible styles
- Use `focus-visible:outline` in Tailwind (already in button.tsx)
- Ensure sufficient contrast between focused and unfocused states

### Keyboard Event Handlers
```tsx
function handleKeyDown(event: React.KeyboardEvent) {
  if (event.key === 'Enter' || event.key === ' ') {
    // Handle activation
    handleAction();
    event.preventDefault();
  }
  if (event.key === 'Escape') {
    // Handle dismissal
    handleClose();
  }
}

<div
  role="button"
  tabIndex={0}
  onClick={handleAction}
  onKeyDown={handleKeyDown}
>
  Action
</div>
```

## ARIA Labels and Semantic HTML

### Use Semantic HTML
```tsx
// Good
<button>Click me</button>
<a href="/page">Link</a>
<header><nav>Menu</nav></header>
<main>Content</main>
<article>News Item</article>
<section>Features</section>
<footer>Footer</footer>

// Avoid
<div onClick={click} role="button">Click me</div> // Use <button>
<span onClick={navigate}>Link</span> // Use <a>
```

### ARIA Labels for Icons
```tsx
import { Trash2 } from 'lucide-react';

// Good: Icon button with aria-label
<button aria-label="Delete item">
  <Trash2 size={20} />
</button>

// With tooltip
<button
  aria-label="Delete item"
  title="Delete item"
>
  <Trash2 size={20} />
</button>
```

### Form Labels
```tsx
// Good
<label htmlFor="email">Email</label>
<input id="email" type="email" />

// Using aria-label when label not visible
<input
  type="search"
  placeholder="Search products"
  aria-label="Search products by name or ID"
/>

// Grouping related inputs
<fieldset>
  <legend>Product Information</legend>
  <label htmlFor="name">Product Name</label>
  <input id="name" type="text" />
</fieldset>
```

### Descriptive ARIA
```tsx
// Loading state
<div role="status" aria-label="Loading products">
  <LoadingSpinner />
</div>

// Progress indicator
<div
  role="progressbar"
  aria-valuenow={progress}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-label="Upload progress: 45%"
>
  {/* progress bar */}
</div>

// Modal dialog
<div
  role="dialog"
  aria-labelledby="dialog-title"
  aria-describedby="dialog-description"
>
  <h2 id="dialog-title">Confirm Action</h2>
  <p id="dialog-description">Are you sure?</p>
</div>

// Alerts and errors
<div role="alert" aria-label="Error">
  Please fill in all required fields
</div>
```

## Color Contrast

### Contrast Requirements
- **Normal text**: 4.5:1 ratio (WCAG AA)
- **Large text** (18pt+): 3:1 ratio (WCAG AA)
- **UI components**: 3:1 ratio

### Testing Contrast
- Use [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- Use browser DevTools accessibility inspector
- Run in your CI/CD pipeline

### Good Contrast Examples
```css
/* Good */
.text-primary {
  color: #000080; /* Dark blue on white: 9.3:1 */
}

.text-secondary {
  color: #555555; /* Gray on white: 8.59:1 */
}

.text-disabled {
  color: #999999; /* Light gray on white: 3.85:1 (borderline) */
}

/* Avoid */
.text-bad {
  color: #BBBBBB; /* Light gray on white: 2.42:1 ❌ */
}
```

## Screen Reader Support

### Meaningful Images
```tsx
// Good: Descriptive alt text
<img
  src="/product-image.jpg"
  alt="Organic coffee beans in a white ceramic cup"
/>

// Decorative images
<img src="/decorative-line.svg" alt="" /> {/* Empty alt for decorative */}

// Icons
<svg aria-label="Location">
  <use href="#location-icon" />
</svg>
```

### Dynamic Content Updates
```tsx
// Announce changes to screen readers
<div
  role="status"
  aria-live="polite"
  aria-atomic="true"
>
  {successMessage}
</div>

// For loading states
<div
  role="status"
  aria-live="assertive"
  aria-label="Loading products"
>
  {isLoading && <LoadingSpinner />}
</div>
```

### Skip Links
```tsx
// Add skip link at start of page
<a href="#main-content" className="sr-only focus:not-sr-only">
  Skip to main content
</a>

{/* Page content */}
<main id="main-content">
  {/* Content here */}
</main>

// sr-only class
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

.sr-only:focus {
  position: static;
  width: auto;
  height: auto;
  padding: inherit;
  margin: inherit;
  overflow: visible;
  clip: auto;
  white-space: normal;
}
```

## Component Accessibility Checklist

### Loading States
- ✅ Use `role="status"` for loading indicators
- ✅ Announce "Loading..." with aria-label
- ✅ Show indeterminate progress for unknown duration
- ✅ Use `aria-live="polite"` for status updates

### Forms
- ✅ All inputs have associated labels (via `<label>` or `aria-label`)
- ✅ Required fields marked with `aria-required="true"`
- ✅ Error messages linked with `aria-describedby`
- ✅ Grouped inputs in `<fieldset>` with `<legend>`

### Modals
- ✅ Focus trapped within modal
- ✅ Labeled with `aria-labelledby` and `aria-describedby`
- ✅ Marked with `role="dialog"` or `aria-modal="true"`
- ✅ Can be closed with Escape key

### Navigation
- ✅ Current page indicated with `aria-current="page"`
- ✅ Navigation regions marked with `<nav>` or `role="navigation"`
- ✅ Keyboard accessible menu with arrow keys
- ✅ All links have descriptive text

### Tables
- ✅ Marked with `<table>` with proper `<thead>`, `<tbody>`
- ✅ Row and column headers marked with `scope` attribute
- ✅ Captions with `<caption>`
- ✅ Complex tables marked with `aria-describedby`

## Testing for Accessibility

### Automated Testing
```bash
# Lighthouse
npm run lighthouse

# axe DevTools
# https://www.deque.com/axe/devtools/

# WAVE
# https://wave.webaim.org/

# Pa11y
npx pa11y https://localhost:3000
```

### Manual Testing
1. **Keyboard only**: Navigate entire site using only Tab, Shift+Tab, Enter, Space, Arrow keys
2. **Screen reader**: Test with NVDA (Windows), JAWS, or VoiceOver (Mac/iOS)
3. **Color blindness**: Verify without relying solely on color
4. **Zoom**: Test at 200% zoom
5. **Reduced motion**: Respect prefers-reduced-motion

### Browser Tools
- Firefox Accessibility Inspector (press F12, Accessibility tab)
- Chrome DevTools Accessibility Audit
- Safari Web Inspector Accessibility

## Common Mistakes to Avoid

❌ **Don't**:
- Remove focus outlines without replacing them
- Use `position: absolute` with negative margins as hide mechanism (breaks screen readers)
- Put all content in images without alt text
- Rely only on color to convey information
- Auto-play audio or video
- Use placeholder text as label
- Create keyboard traps

✅ **Do**:
- Test with keyboard navigation
- Provide skip links
- Use semantic HTML
- Include meaningful alt text
- Provide visible focus indicators
- Label all form inputs
- Test with screen readers

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [MDN Accessibility](https://developer.mozilla.org/en-US/docs/Web/Accessibility)
- [WAI-ARIA](https://www.w3.org/WAI/ARIA/apg/)
- [Deque Academy](https://dequeuniversity.com/)
- [WebAIM](https://webaim.org/)

## Contributing

When contributing to ChainLogistics:
1. Follow these accessibility guidelines
2. Test keyboard navigation on your changes
3. Run accessibility audits (Lighthouse, axe)
4. Request accessibility review in PRs
5. Update this guide if adding new patterns

