<p align="center"><img src="public/brand/pukaar-logo-color.svg" alt="Pukaar" height="120"></p>

<h1 align="center">Pukaar</h1>

**A community issue tracker where residents confirm the fix.**

*Pukaar* means a call, a cry out. Here, a call stays on the record until the people who made it agree it was answered. A steward can say the streetlight is fixed, but only the person who reported it, or two other residents, can make that true. Nobody can quietly rewrite the history afterward, and no server holds the only copy.

Pukaar is a [Logos Basecamp](https://github.com/logos-co/logos-basecamp) app for a shared site such as a camp, a building or a neighborhood. Members report faults, follow each one on an Issue Tracker, and talk it through in the same app. Built for the Field Station civic reporting brief.

[![ci](https://github.com/adysingh5711/pukaar/actions/workflows/ci.yml/badge.svg)](https://github.com/adysingh5711/pukaar/actions/workflows/ci.yml)
![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)
![status](https://img.shields.io/badge/status-pilot--ready%20v0.4.2-blue)

| At a glance | |
|---|---|
| Who it is for | Residents, stewards and admins of a small shared site. The sample site is Dhun Camp |
| What it replaces | The spreadsheet, group chat or paper log where whoever runs it decides what "resolved" means |
| What you can rely on | Nothing is ever deleted, and there is no server to run |
| How it syncs | Peer to peer over Logos Delivery. Every member holds a full copy. Anchors on the Logos Execution Zone (LEZ) fix the history |
| What it costs today | Nothing to run. See [What it costs](docs/install-and-run.md#what-it-costs) |
| Runs on | Logos Basecamp 0.3.1 or higher |
| Latest release | [v0.4.2](https://github.com/adysingh5711/pukaar/releases/tag/v0.4.2): `.lgx` packages for Logos Basecamp 0.3.1 |
| Demo video | To be added |

## See it

<table>
  <tr>
    <td align="center"><a href="public/images/01-board.png"><img src="public/images/01-board.png" alt="The Issue Tracker with four columns of reports" width="280"></a><br>Issue Tracker</td>
    <td align="center"><a href="public/images/02-why-not-closed-yet.png"><img src="public/images/02-why-not-closed-yet.png" alt="A new report with the three steps to closure" width="280"></a><br>Why not closed yet</td>
    <td align="center"><a href="public/images/03-confirm-the-fix.png"><img src="public/images/03-confirm-the-fix.png" alt="A claimed fix with Confirm and Reopen" width="280"></a><br>Confirm the fix</td>
  </tr>
  <tr>
    <td align="center"><a href="public/images/04-history.png"><img src="public/images/04-history.png" alt="History of resolved and closed issues by month" width="280"></a><br>History</td>
    <td align="center"><a href="public/images/05-report-a-fault.png"><img src="public/images/05-report-a-fault.png" alt="The report form with a list of places" width="280"></a><br>Report</td>
    <td align="center"><a href="public/images/06-profile.png"><img src="public/images/06-profile.png" alt="The Profile page with identity backup and invite" width="280"></a><br>Profile</td>
  </tr>
  <tr>
    <td align="center"><a href="public/images/07-people.png"><img src="public/images/07-people.png" alt="The People page with roles and approvals" width="280"></a><br>People</td>
    <td align="center"><a href="public/images/08-places-change-log.png"><img src="public/images/08-places-change-log.png" alt="The Places page with its change log" width="280"></a><br>Places</td>
    <td align="center"><a href="public/images/09-proof.png"><img src="public/images/09-proof.png" alt="The Integrity page with the last anchor status" width="280"></a><br>Integrity</td>
  </tr>
</table>

Each screen is explained in [Screenshots](docs/screenshots.md).

## A fault's life

Picture a tap that drips in the shared washroom of a camp.

1. A resident reports it. It lands on the Issue Tracker as open.
2. A steward acknowledges it and posts the next step and an ETA.
3. The steward fixes the tap and claims the fix, with a note. A claim is not a closure.
4. The reporter confirms, or two other residents do. The claimant never can. The reporter can always reopen.
5. Each step is a signed event in its author's own log. Nothing is deleted.

<p align="center"><img src="public/images/anatomy-two-key-closure.svg" alt="Two-key closure: a steward claims the fix, then the reporter or two other residents confirm it." width="720"></p>
<p align="center"><em>A steward claims the fix. The reporter, or two other residents, turn the second key.</em></p>

The full story and the state diagram are in [How it works](docs/how-it-works.md#a-faults-life).

## Who does what

| Role | What they do | Limit |
|---|---|---|
| Resident | Reports faults, confirms or reopens a fix, chats and meets | Can be named by the admin, or stay pseudonymous |
| Steward | Acknowledges, updates, claims a fix with a note, closes without a fix, marks duplicates | Always named. Never confirms their own claim |
| Admin | Does what a steward does. Approves people, grants roles with a reason everyone can see, manages places | Cannot grant or change admins |
| Super admin | Grants and changes admins. At most two per site | Nobody can revoke a super admin |

The full table is in [Features and roles](docs/features.md#who-does-what).

## Talk it through

The tap is still dripping, and the reporter wants to ask the steward a question. Pukaar has private chat (one to one and in groups) and meetings for up to eight people built in. The talk about a fault stays with the site, and nobody runs a server.

<p align="center"><img src="public/images/anatomy-logos-stack.svg" alt="The Logos modules under Pukaar: Delivery, Chat, Storage and LEZ." width="720"></p>
<p align="center"><em>Each part of Pukaar sits on a Logos module.</em></p>

| Part | Logos module |
|---|---|
| Moving messages and event sync | `delivery_module` 0.4.0 |
| Private chat | `chat_module` 0.3.0 |
| Meeting messages | `delivery_module` plus Pukaar's own seal |
| Meeting backup | `storage_module` 3.0.2 |
| History checkpoints and backup pointer | LEZ (`pukaar_registry`) |

Usage steps, the full table and the limits are in [Features and roles](docs/features.md#talk-it-through-in-the-same-app).

## Install (v0.4.2)

Add this source in Basecamp, Package Manager: `https://raw.githubusercontent.com/adysingh5711/pukaar/main/public/install/logos-repo.json`. You can also use the [install page](https://pukaar-logos.vercel.app/), or install by hand with the release notes. The packages need Logos Basecamp 0.3.1 on a Mac with Apple silicon, or on 64-bit Linux (Intel, AMD or ARM). Intel Macs and Windows are not supported.

The full steps, network notes and profile tips are in [Install and run](docs/install-and-run.md#run-pukaar-in-basecamp). The release notes are on the [v0.4.2 release page](https://github.com/adysingh5711/pukaar/releases/tag/v0.4.2).

## Go deeper

| Document | Open it to |
|---|---|
| [How it works](docs/how-it-works.md) | See how events, sync, checkpoints and the chain registry fit together |
| [Features and roles](docs/features.md) | Know every feature, every role and the chat and meeting details |
| [Screenshots](docs/screenshots.md) | Read what each of the nine screens shows |
| [Install and run](docs/install-and-run.md) | Install Pukaar, run several profiles, upgrade or see the cost |
| [Security](docs/security.md) | Study the threat model, or confirm a signature or an anchor |
| [Developing](docs/developing.md) | Run the tests, build from source or cut a release |
| [Roadmap](docs/roadmap.md) | See what shipped and what comes next |

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

## Acknowledgements

Built on [Logos](https://logos.co) (Basecamp, Delivery, LEZ, SPEL and `logos-scaffold`) for the Field Station civic reporting brief.
