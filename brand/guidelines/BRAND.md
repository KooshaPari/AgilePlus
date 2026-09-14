# AgilePlus Brand Guidelines

## Brand Identity

**AgilePlus** is a spec-driven development platform. The brand conveys precision, velocity, and clarity.

---

## Logo

The AgilePlus mark combines three elements:
1. **Velocity bars** — five ascending/descending bars representing sprint progress
2. **Sprint ring** — a circular arc with an arrow showing continuous iteration
3. **Plus sign** — overlaying the center bars, representing the "Plus" in AgilePlus

### Logo Variants

| Variant | File | Use Case |
|---------|------|----------|
| Full color | `icon/agileplus-logo.svg` | Default on dark backgrounds |
| Monochrome white | `icon/agileplus-logo-white.svg` | On teal/brand backgrounds |
| Monochrome dark | `icon/agileplus-logo-dark.svg` | On light backgrounds |
| Favicon | `icon/agileplus-favicon.svg` | Browser tabs, small spaces |

---

## Color Palette

### Primary
| Name | Hex | RGB | Use |
|------|-----|-----|-----|
| Teal Bright | `#5ecec6` | 94, 206, 198 | Highlights, active states |
| Teal Primary | `#2db5ab` | 45, 181, 171 | Primary brand, buttons, links |
| Teal Mid | `#1a9e95` | 26, 158, 149 | Secondary elements |
| Teal Deep | `#157e77` | 21, 126, 119 | Borders, dividers |
| Teal Dark | `#0f6962` | 15, 105, 98 | Shadows, depth |

### Backgrounds
| Name | Hex | Use |
|------|-----|-----|
| BG Primary | `#0a1a18` | Main app background |
| BG Surface | `#0c2420` | Cards, panels |
| BG Elevated | `#0f2e2a` | Modals, popovers |
| BG Deep | `#060e0d` | Deepest background layer |

### Text
| Name | Hex | Use |
|------|-----|-----|
| Text Primary | `#e8f8f6` | Headings, primary text |
| Text Secondary | `rgba(160,220,215,0.6)` | Body text, descriptions |
| Text Tertiary | `rgba(160,220,215,0.35)` | Captions, hints |
| Text Ghost | `rgba(160,220,215,0.12)` | Decorative text overlays |

### Semantic
| Name | Hex | Use |
|------|-----|-----|
| Success | `#4caf50` | Passed, complete |
| Warning | `#ff9800` | In progress, review needed |
| Error | `#f44336` | Failed, blocked |
| Info | `#2196f3` | Informational |

---

## Typography

### Desktop App
- **System font stack**: `-apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', sans-serif`
- **Monospace**: `'SF Mono', 'Fira Code', 'Cascadia Code', monospace`

### Scale
| Element | Size | Weight | Letter Spacing |
|---------|------|--------|----------------|
| H1 | 28px | 600 | -0.3px |
| H2 | 22px | 600 | -0.2px |
| H3 | 17px | 600 | 0 |
| Body | 14px | 400 | 0 |
| Caption | 13px | 400 | 0.3px |
| Label | 11px | 500 | 3px (uppercase) |

---

## Iconography

Follows Phenotype org iconography standard:
- **Style**: Material Design outline
- **Viewbox**: 24x24
- **Stroke**: 2px, round caps/joins
- **Color**: `currentColor`
- **Accessible**: `role="img"`, `aria-label`, `<title>`, `<desc>`

### Icon Inventory
| Icon | Source |
|------|--------|
| `arrow-right` | `fluent/arrow-right.svg` |
| `check-circle` | `fluent/check-circle.svg` |
| `home` | `fluent/home.svg` |
| `link` | `fluent/link.svg` |
| `plus` | `fluent/plus.svg` |
| `search` | `fluent/search.svg` |
| `settings` | `fluent/settings.svg` |
| `terminal` | `fluent/terminal.svg` |
| `user` | `fluent/user.svg` |
| `x-circle` | `fluent/x-circle.svg` |

---

## Visual Language

### Panel Art
- Dark backgrounds with teal ambient glow
- Sprint rings (subtle rotating circles) for depth
- Velocity bars (ascending/descending rectangles) as a recurring motif
- Corner accent lines on panels
- Minimal text overlays in ghost opacity

### Animations
- **Loading**: Velocity bars bounce in sequence + spinning sprint ring
- **Transitions**: 200ms ease-out for state changes
- **Hover**: Subtle brightness increase (5-10%)

### Gradients
- Logo body: `linear-gradient(155deg, #5ecec6, #2db5ab, #1a9e95, #157e77, #0f6962)`
- Specular: `linear-gradient(140deg, rgba(255,255,255,0.85), transparent 25%)`
- Ambient glow: `radial-gradient(circle, rgba(46,181,171,0.15), transparent 70%)`

---

## Application

### Desktop App (Tauri)
- Window background: `#0a1a18`
- Title bar: transparent (native)
- Sidebar: `#060e0d`
- Content cards: `#0c2420`
- Active item: `rgba(45,181,171,0.12)` background, `#2db5ab` left border

### README / GitHub
- Badge style: flat, teal background
- OpenGraph card: 1200x630, dark bg with logo + text

---

## Design Principles

1. **Dark-first** — all UI designed for dark mode first
2. **Teal accent** — brand color used sparingly for focus
3. **Depth through layers** — background > surface > elevated
4. **Velocity motif** — bars appear in icons, animations, decorations
5. **Minimal chrome** — content is the focus, UI recedes
