**Pukaar v0.4.0** for **Logos Basecamp 0.3.1**. Packages for Mac with Apple silicon (`darwin-arm64`).

## What's new since v0.3.0
- **Chain port:** Pukaar now writes to the Logos Execution Zone (LEZ) through its SPEL program. An **Anchor now** button records the state of the event log on the chain. The chain also keeps one backup pointer for each site and member. If the account cannot pay for a failed write, the write waits, and the balance is shown.
- **Profiles:** each member has a profile with a bio, experience and up to four links. A privacy switch hides these from other members. A member page shows the profile, and People rows have a Message action.
- **Meetings and backup:** 1:1 meetings with time slots, clash warnings and reminders. Meeting data is backed up in encrypted form on Logos Storage. The chain pointer lets you restore it.
- **Private chat with groups:** members can talk in private, one to one or in groups. A group has a member list. Unread dots show where new messages wait, on the Messages tab and on chat rows.
- **Notices panel:** a Notifications panel holds the app notices. Start-up notices wait 3 seconds, and info notices expire.
- **Install page:** `public/install/index.html` explains both install ways. It also holds the source file for Basecamp.

## Install
Choose one way. Then fully quit Basecamp and start it again. Closing the window is not enough.

**Way 1: add a source.** In Basecamp, open Package Manager, choose Add source, and paste:
```
https://raw.githubusercontent.com/adysingh5711/pukaar/main/public/install/logos-repo.json
```
Choose Pukaar and Install. Accept the modules it asks for.

**Way 2: by hand.** In Package Manager, choose Install Local Package for each file, in this order:
1. `chat_module-0.3.0.lgx` (from the Logos module catalog)
2. `logos-pukaar_core-module-lib.lgx`
3. `logos-pukaar_ui-module.lgx`

If Basecamp does not have `delivery_module` 0.4.0 and `storage_module` 3.0.2, install them first. Your existing data is kept.

Check your download with `shasum -a 256 -c SHA256SUMS`.

## What you need
- Logos Basecamp 0.3.1.
- A Mac with Apple silicon. Other systems are not supported yet.
- `delivery_module` 0.4.0.
- `chat_module` 0.3.0.
- `storage_module` 3.0.2. We have not tested this version with Pukaar.

## Signature check
The publisher DID is:
```
did:jwk:eyJjcnYiOiJFZDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6IlpfZkxKcnVWR3UyYnBiR0VNMlhMTElmY2FzdTFycVkycHJZM1Z0cklRR28ifQ
```
To check the packages, run these commands:
```
lgx keyring add publisher "<DID above>" --dir ./trusted-keys
lgx verify logos-pukaar_core-module-lib.lgx --keyring-dir ./trusted-keys
lgx verify logos-pukaar_ui-module.lgx --keyring-dir ./trusted-keys
shasum -a 256 -c SHA256SUMS
```
The official module index ships `"trustedSigners": []`, so Basecamp does not enforce signatures yet. Run these checks yourself.

## Known limits
- A group join can take several minutes.
- A person who joins a group later does not see the earlier messages.
- A message sent to an old address is not sent again.
- We have not tested the chat screens live.
- In a new chat, the first message waits about 10 seconds.

Licence: MIT OR Apache-2.0.
