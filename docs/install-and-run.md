# Install and run

How to install Pukaar in Basecamp, run several profiles, upgrade from v0.2.4, what it costs and what you need first. [Back to the README](../README.md).

## Run Pukaar in Basecamp

1. Download `pukaar-core.lgx` and `pukaar-ui.lgx` from the [latest release](https://github.com/adysingh5711/pukaar/releases/latest), and check them (see [Security](security.md)).
2. In Basecamp 0.3.1: **Package Manager → Install Local Package**. Install the core package first, then the UI package.
3. Open Pukaar. Everyone except the admin pastes the site id under **Join a site**, then reads their fingerprint aloud at the kiosk so the admin can approve them. The admin opens **Create a site** instead, fills in the form, and clicks **Create site**. **Restore your identity** brings back an exported identity on a new install.

Pukaar talks over Delivery's `logos.dev` network, which needs no RLN membership. Every device of a site must run a Pukaar build on the same network: builds before this one used `logos.test`, where Delivery 0.3.0 sends nothing until the profile holds a funded on-chain RLN membership. After installing a new Pukaar, quit Basecamp fully and start it again: Delivery keeps the network it was first started with until then.

If the sidebar says **offline: messages aren't reaching the network**, Delivery takes Pukaar's messages but can't get them out. Check Basecamp's log for `Failed to send message`.

One Basecamp profile holds one Pukaar identity. To run several people on one machine, give each its own Basecamp data directory, side by side:

```bash
scripts/basecamp.sh default    # your normal Basecamp profile
scripts/basecamp.sh asha       # a separate profile (install Pukaar once in it)
```

`scripts/basecamp.sh` works on macOS only, because it starts the binary inside a `.app` bundle. It uses `/Applications/LogosBasecamp.app`, and `BASECAMP_APP` sets another bundle path. The `default` profile is `~/Library/Application Support/Logos/LogosBasecamp`. Each named profile is a directory in `PUKAAR_BASECAMP_DIR`, and the default is `~/.local/share/pukaar-basecamp`.

## Upgrade from v0.2.4

<details>
<summary>Upgrade steps</summary>

v0.3.0 adds new event types, and a v0.2.4 app cannot decode them. Each event links to the previous event of its author. Thus a v0.2.4 app also holds back all later events from a member who signs a new event. The existing data of a site stays. Pukaar makes the site creator the first super admin from the existing history, without a new event. The test `a_v0_2_4_log_replays_to_the_same_state` replays a v0.2.4 log (`tests/fixtures/v0_2_4_log.json`) and gets the same state.

1. In each Basecamp profile, install the core package of the latest release, then the UI package.
2. Quit Basecamp fully, then start it again.
3. Tell every member of the site to do the same steps.

</details>

## What it costs

A site pays nothing to run Pukaar today. It needs these items:

- A laptop for each member, with [Logos Basecamp 0.3.1](https://github.com/logos-co/logos-basecamp/releases/tag/0.3.1).
- One kiosk laptop, if the site wants one.
- An internet link. A relay for sites with no internet is not built yet.

These items are free today:

- Messages go over `logos.dev`, a Logos Delivery network for development. It needs no account, no domain and no database, and it charges no fee.
- An anchor puts a short summary of the site history on the LEZ chain. Anchors on localnet, a private test chain, are free. Localnet loses these anchors when it stops.

These items can cost money later. Their price is not known yet:

- On `logos.test`, the network for production sites, each laptop needs an RLN membership, a paid pass to send a set number of messages. The site can pay for all members.
- An anchor on a public LEZ network pays a small fee in the LEZ token, the coin of that chain.

Sync also uses data. In a site of 50 members, each laptop receives about 116 MB a day. In a site with four times as many members, each laptop receives about 16 times as much. The team keeps the technical cost notes in a separate report.

## Prerequisites

- Rust 1.96+ (`rustup`)
- For the chain program: Docker, [`logos-scaffold`](https://github.com/logos-co/scaffold) (`cargo install logos-scaffold`, which provides `lgs`), and the RISC Zero toolchain (`rzup install rust`, `rzup install r0vm`)
- For the Basecamp modules: Nix with flakes, and [Logos Basecamp 0.3.1](https://github.com/logos-co/logos-basecamp/releases/tag/0.3.1)
- Hindi text in reports uses the system's Devanagari font, so a bare Linux kiosk should install `fonts-noto-core`
