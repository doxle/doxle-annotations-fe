# Annotations Block

## Architecture

**Atoms** = Reusable, block-agnostic
- `atoms/drawing/svg_canvas/` — Canvas renderer (pan, zoom, render Geometry)
- `atoms/drawing/` owns `Geometry`, `Point` — shared primitives

**Blocks** = Domain-specific
- `blocks/annotations/` — Owns Annotation, Label, API calls, hit-testing

## Why This Split?

The same canvas atom gets reused by different blocks:
- **Annotations** — Labels, hit-testing for edit/delete
- **Building** — Budgets, takeoffs (no labels)
- **Photos** — Markup tool

Canvas is a dumb renderer. It receives shapes (`Geometry`) and emits mouse events.
Block handles business logic (save, validate, domain models).

Tomorrow we can swap `svg_canvas` for `bevy_canvas` (3D) or `blitz_canvas` (perf) — blocks don't change.

## This Block Owns

- `Annotation` model
- `Label` model
- API calls to `/annotations/*`
- Hit-testing (right-click edit/delete)
- Sidebar (labels list)
