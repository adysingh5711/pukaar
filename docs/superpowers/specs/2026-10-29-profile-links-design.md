# Profile links and profile presentation

## Goal

Bring the QML profile pages into parity with the repository mockups while preserving the existing profile data model, privacy rules, and external-link confirmation flow.

## Scope

- Use the `i-panel` collapse glyph from `design/01-signal-desk.html` for the sidebar control.
- Add the mockup's social icons for GitHub, LinkedIn, X, Website, and Other.
- Show those icons in the current user's editable link rows and in other members' profile link chips.
- Keep each member link as a chip. Clicking a chip opens an inline confirmation row with:
  - `Open link`: calls `Qt.openUrlExternally("https://" + address)`.
  - `Copy URL`: copies the full HTTPS URL and briefly reports `Copied URL` on the action.
  - The existing trust warning remains visible before opening.
- Add a no-image avatar placeholder to the current user's Profile header, matching the existing member-profile avatar placeholder. No image files or remote image loading are added.
- Rework the current user's Profile header and social-link area to use the same visual language as `06-harbor.html`: profile identity first, icon-bearing link inputs, and compact grouped sections.

## Design and data flow

The existing `socials` representation remains `[label, address]`, with addresses stored without a scheme. A small QML mapping supplies the display label and icon name; it does not alter what the core stores or validates. The copy action prepends `https://` only for the clipboard value and browser target.

The existing `root.copyText()` helper remains the clipboard boundary. Link confirmation state is local to `memberPage`, so opening one chip closes the prior confirmation and does not leak state between profiles. Copy feedback is transient and accessible through the button label and accessible name.

The icon set is represented by the existing QML `Icon` component and local path data, using the SVG path shapes already established in the HTML mockups. No new runtime dependency is needed.

## Privacy and safety

The current `memberPage.full` gate remains the only condition that exposes bio, experience, and links. Removed, private, and steward profiles continue to show only the allowed identity information. External URLs remain constrained by the core's existing social-label and address validation; the UI always uses the HTTPS-prefixed value for opening and copying.

## Verification

- `git diff --check`.
- Build the UI portable package with `nix build --accept-flake-config .#lgx-portable` from `modules/pukaar_ui`.
- Confirm one icon mapping exists for every supported social label.
- Confirm the member link confirmation includes both Open and Copy actions and that the copy action uses the full HTTPS URL.
- Confirm the own-profile header contains a placeholder avatar and link editors remain bounded to four entries.

## Out of scope

- User-uploaded images, image persistence, or new profile fields.
- Changing the core profile schema or link validation.
- Replacing the browser-opening behavior with copy-only behavior.
