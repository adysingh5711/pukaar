# Features and roles

The full feature list, the role table and the chat and meeting details, for readers who want to know what members get. [Back to the README](../README.md).

## Why

Taking in a fault report is easy. Closing it is where fault reporting fails, because whoever runs the system decides what counts as "resolved". In FixMyStreet, the best-known open-source tool, whether the public can even reopen a closed report is a per-council setting.

Pukaar takes that power away from the operator. The site keeps the whole record, and the same rules apply to everyone.

| What you get | How it works |
|---|---|
| A report nobody can delete | Every member holds a full replica. There is no delete event, action or button anywhere. An invalid event is kept, marked `rejected`, and shown. |
| A fix that needs your word | A steward can claim a fix. Only the reporter, or two distinct residents who are not the claimant, can confirm it. The reporter can always reopen. |
| Names you can see | Stewards and admins are always named, by the admin's grant. Residents may be named by the admin when approved, and can replace that name or clear it to stay pseudonymous. Every staff update and fix claim must say what was done. |
| A history that stays put | Per-author hash chains expose any rewrite (a fork proof). Checkpoints anchored on LEZ fix the history even for people who join later. |
| One app for the site | The Issue Tracker, private chat with groups, meetings and a Home page that shows what waits for you all live in the same app. There is no server to run. |

## Who does what

| Role | What they do | Limits |
|---|---|---|
| Resident | Reports faults and follows them. Confirms a fix as the reporter, or as one of two residents who did not claim it. Reopens a fix that did not hold. Chats and meets other members. | May be named by the admin, and can replace that name or stay pseudonymous. A member who waits for approval cannot report, meet or message yet. |
| Steward | Acknowledges reports, posts updates with the next step and an ETA, claims a fix with a note, closes as won't-fix with a reason, and marks duplicates. | Always named. A claim is not a closure, and a steward can never confirm their own claim. |
| Admin | Does everything a steward does. Approves each person after hearing their fingerprint read aloud, grants and revokes roles with a reason everyone in the site can see, and manages the site's places. | Cannot grant or change admins. Cannot change a super admin. |
| Super admin | Grants and changes admins. Moves an admin to steward, or removes them, after a notice (now, 1, 7 or 30 days, or a custom time). Can add a second super admin, remove the other one, or transfer the role. | The site creator's key is the first super admin. A site has at most two. Nobody can revoke a super admin or change their role. |

## Features

| Area | What members get |
|---|---|
| Report and track | Home page, Issue Tracker, a report form with a grouped places list, the staff workflow and two-key closure |
| People and roles | Join with the site id, fingerprint approval, named staff, super admins with admin notices, profiles |
| Talk | Private chat, one to one and in groups, with unread dots. Meetings for up to eight people with free or busy times, reminders and an encrypted backup. See [Talk it through](#talk-it-through-in-the-same-app) |
| Notices | A notifications panel |
| Places | A table of places with status and open issues, plus a searchable change log |
| Safety | Identity backup, the Integrity page, nothing ever deleted |

The full list is below.

<details>
<summary>All features</summary>

- **Home:** the first page after you open a site. It greets you, shows the site name, counts open and recently fixed issues, and lists what waits for you: unread chats, meeting requests, issues past their target (staff), your open reports, and people waiting for approval (admin).
- **Issue Tracker (the page that was called Board):** every open report in four columns, from reported to fixed, with a switch for only your reports.
- **Report** what's wrong and where: a category, one line, and a place picked from the site's grouped, searchable location list (or "Other" with a description).
- **Staff workflow:** acknowledge, post updates with the next step and an ETA, claim a fix (a note is required), close as won't-fix, or mark a duplicate. Overdue and waiting-for-48 h cards are highlighted. Actions that cannot be undone ask for confirmation first, and a disabled button says why it is disabled.
- **Two-key closure:** only the reporter, or two residents who aren't the claimant, can confirm a fix. The reporter can always reopen.
- **People:** join with the site id; the admin approves each person after hearing their fingerprint read aloud. Staff are always named by the admin; the admin may name a resident too, and residents can change that name or stay pseudonymous. Every member sees who has which role (residents and stewards read-only) and a Removed list that says who removed each person, when and why. Admins can revoke a role, with a reason everyone in the site can see asked in a confirm step, and can "Grant again". The removed person's app says who removed them and why, and their past reports and timeline entries are marked "former member".
- **Super admins and admin notices:** the site creator's key is the first super admin. A site has at most two. A super admin can add a second super admin, remove the other one, or transfer the role. Nobody can revoke a super admin or change their role. Only super admins grant or change admins. A super admin can move an admin to steward or remove them after a notice (now, 1, 7 or 30 days, or a custom time). A super admin can also cancel a notice. A deadline in the past counts as now. The admin keeps admin powers until the deadline. At the deadline every peer treats that person as the new role and rejects their later admin actions. A staff member's app seals the notice by itself 10 minutes after the deadline, which also defeats backdated events.
- **Private chat with groups:** members message each other one to one or in groups, with a member list, unread dots on the Messages tab and on chat rows, and a Message action on People rows and member pages. A group recovers when a member restarts. A person who joins a group later does not see earlier messages.
- **Meetings:** propose a time to one member or up to eight. Each person answers on their own. A meeting is confirmed when everyone has answered and at least one accepts. Set a custom length, start time and reminder lead, see who is free or busy, add a description, and leave a meeting. Meeting data is backed up in encrypted form on Logos Storage, and the chain keeps a pointer so you can restore it.
- **Notifications panel:** app notices wait in one panel that closes when you click outside it. Info notices expire.
- **Navigation:** Home, Issue Tracker, Report, History, People, Messages, Meetings and Places for every member. The admin also sees a count of people waiting on People. A member who waits for approval cannot report, meet or message yet, and sees one banner in place of those controls.
- **Site setup:** the admin creates a site from a guided form: site name, their own name, editable categories, and a places table that starts empty (or from the Dhun sample). Service rules (acknowledge and fix targets in hours, open reports per person) are plain fields with explanations.
- **Places:** every member sees the places with their status and open issues, and a change log they can search by place, person, action or reason. Only the admin adds places and can **edit** a place's name or group (its code never changes; old issues show "renamed from …"). A place in use can be **retired** (blocked while it has open issues), and a retired place offers "Report as Other at this spot". A place nobody ever reported can be **removed**: it shows as pending for 30 days with Undo, then is hidden from every list. The change log lists every place action with who did it, when and why. A duplicate code is refused, and the core rejects place changes from anyone but the admin.
- **Profile:** the "you" block in the sidebar opens a page with your name, your identity backup, the site id to invite someone, your reports and "Leave this site". Each member also has a profile with a bio, experience and up to four links. A privacy switch hides these from other members.
- **Identity backup:** on the Profile page, export a password-sealed copy of your identity (Argon2id + XChaCha20-Poly1305) and restore it on a new install. A restored identity waits for its own history before it can publish, so it can't fork its own chain.
- **Integrity (formerly Proof):** compute a checkpoint of everyone's history and anchor it on LEZ with the printed `spel` command. Covered events show who recorded the anchor. The Integrity page shows when the last anchor was recorded and what it covers, and warns when there is none, when it is older than 24 hours, or when this device cannot rebuild its root. The app does not read LEZ itself, so it never says an anchor is verified (see [Verifying an anchor yourself](security.md#verifying-an-anchor-yourself)).
- **Nothing is ever deleted:** rejected actions are kept and shown. "Removed" places are only hidden; their signed events stay in everyone's log. A forked chain is flagged, and every replica settles on the same branch.
- **One identity, one device:** your key is your identity, with one membership and one role for you in each site. Run it on one device at a time; a restored identity waits for its own history before it can act.

</details>

## Talk it through, in the same app

The tap is still dripping, and the reporter wants to ask the steward a question. Pukaar has private chat and meetings built in, so the talk about a fault stays with the site. Members need no separate chat app or calendar tool, and nobody runs a server.

### How a member uses it

1. Open **Messages** and choose **New chat**, then pick a member. You can also choose **Message** on that person's row in People or on their member page.
2. For a group, choose **New group**, give it a name and pick the members. Each member joins within a minute or two.
3. Write your message. A new chat holds the first message for about ten seconds while the other side joins. A message that cannot go yet shows as waiting and goes out later.
4. To meet, open **Meetings** and choose one to eight members. Pick a length and a free time, add a short description if you like, and send the proposal. **1:1 meeting** on a People row does the same for one person.
5. Each invitee accepts or declines on their own. The meeting is confirmed once everyone has answered and at least one person accepts. A reminder shows before the start, while the app is open.

### What members get

| Feature | What it does |
|---|---|
| One to one chat | Message any active member. The Messages tab lists chats that have messages, with a preview and a time |
| Groups | A named chat with a member list. A group recovers when a member restarts |
| Unread dots | Shown on the Messages tab and on chat rows. Home lists the chats that wait for you |
| Meetings | One organizer and one to eight invitees. Custom length, start time and description (up to 500 characters) |
| Free or busy | You see who is free or busy before you choose a time. Only times are shared, never titles. You can share with the site or with nobody |
| Clash warning | The app names an overlap with another meeting. You can still send the proposal anyway |
| Reminders | Off, or a lead time you set, shown while the app is open |
| Backup | Your meeting history is encrypted, stored on Logos Storage, and findable from the chain on a new device |

### Why it matters

The talk about a fault sits next to the fault. A resident can ask the steward a question, or the two of them can pick a time to look at the tap, without leaving the app. There is no server to run. If you lose a device, you can restore your meeting history from the encrypted backup.

### Built on Logos

| Part | Logos module | What it does |
|---|---|---|
| Moving messages | `delivery_module` 0.4.0 | Carries chat messages, meeting proposals and answers between members. It also carries the site's event sync |
| Private chat | `chat_module` 0.3.0 | Wraps libchat to give end-to-end encrypted chat, one to one and in groups. Pukaar adds a signed address announcement so members can find each other |
| Meeting messages | `delivery_module` plus Pukaar's own seal | Pukaar signs each proposal and answer and seals it for the receiver. Delivery then carries it, so the site channel does not show who meets whom |
| Meeting backup | `storage_module` 3.0.2 | Holds one encrypted copy of your meeting history, locked with a key from your Pukaar key |
| History and backup pointer | LEZ (`pukaar_registry`) | Anchors history checkpoints. It also keeps one pointer per site and member to the newest backup. Meeting details never go on the chain |

### What is not promised

- A member who joins a group later does not see the earlier messages.
- A group join can take several minutes. A message sent to an old chat address is not sent again.
- Chat needs the Logos key service (`devnet.chat-kc.logos.co`) to start.
- Delivery gives no acknowledgement. A reminder shows only while the app is open.
- Free or busy times go to every member while you share them.
- We have not tested the chat screens live. We have not tested `storage_module` 3.0.2 with Pukaar.
