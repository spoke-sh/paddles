# Modularize React Runtime Application - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-04-08T13:40:00-07:00

- Defined the voyage-one target module map around `app/`, `chat/`, `store/`, and `presentation/` seams instead of splitting the runtime into shallow presentational atoms.
- Declared `selectedManifoldTurnId` a shell-owned cross-route state surfaced through a dedicated chat-owned context because transcript clicks drive manifold selection.
- Declared prompt history, paste compression, and sticky-tail scrolling composer/transcript-local concerns so later route refactors do not absorb them into route modules.

## 2026-04-08T14:20:42

Mission achieved by local system user 'alex'
