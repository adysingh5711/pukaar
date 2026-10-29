# Roadmap

What shipped in earlier releases and what comes next. [Back to the README](../README.md).

<details>
<summary>Earlier releases</summary>

- **L1 (done in v0.1.0):** the full report → claim → confirm loop between instances over Logos Delivery, running inside Basecamp, with signed portable `.lgx` packages.
- **v0.1.1:** guided site setup, edit and remove locations (30-day pending removal), a change log, service rules as form fields, deterministic fork resolution.
- **v0.2.0:** UI revamp: light and dark themes, a collapsible sidebar, redesigned board, cards and issue pane (with a "Why it isn't closed yet" card), a board history window with a History page, and Hindi user content.
- **v0.2.1:** fix: calls no longer hang for 20 s when Logos Delivery is unreachable (5 s Delivery timeouts plus a 10 s back-off), and clearer error text when the core does not answer.
- **v0.2.2:** fix: sync works again (Pukaar turns off Delivery's SDS causal-history wait), moved to the `logos.dev` network, recovery after a Delivery restart, and names an admin gives residents now show on both sides.
- **v0.2.3:** navigation built from the role, a Profile page, an admin Site page with a full place change log, a Proof page with the last-anchor status, issue pane actions above the timeline, confirm steps and reasons for disabled buttons, and revocations that say who removed whom, when and why.
- **v0.2.4:** every member sees People (roles, pseudonyms and who was removed, when and why) and Places (status, open issues and a searchable change log); admin controls stay admin only. Packages now carry their real version (they all said 0.1.0 before), and CI fails when a release tag and the package version disagree.
- **v0.3.0:** History and Places (with its change log) are tables, and reasons are asked in confirm steps. Integrity left the sidebar: a status strip on History opens it, and the Board shows a red banner only on an anchor mismatch. Super admins (at most two, with transfer) are the only ones who grant or change admins, and admin notice periods move an admin to steward or remove them after a notice, sealed by a staff member's app 10 minutes after the deadline. Breaking: v0.3.0 adds new event types, so every member must update. v0.2.4 apps cannot read the new events.
- **v0.4.0:** chain port, profiles, meetings and backup, private chat with groups and unread dots, a notices panel and an install page. See [RELEASE_NOTES_0.4.0.md](../RELEASE_NOTES_0.4.0.md).

</details>

- **L1+:** anchoring from the app's Integrity page on localnet and a testnet anchor; an "identity active on another device" warning; Heads messages that carry head ids, so anti-entropy also repairs equal-length forks.
- **L2:** LAN relay for sites without internet, a password-sealed key file, a site-health view (category level only, no per-person ranking), export and verify bundles, evidence photos on Logos Storage (EXIF stripped), UI tests beyond the screenshot harness, and a steward guide in Hindi and English.
- **L3:** an anonymous reporting lane with an RLN rate limit, a phone path, and 2-of-3 admin grants.

Dates and milestones beyond L1: to be added in later version.
