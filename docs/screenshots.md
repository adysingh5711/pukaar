# Screenshots

The nine screens in the order you meet them, with what each one shows, and how to render them again. [Back to the README](../README.md).

Pukaar's own screens in the order you meet them, rendered offscreen with sample data by `scripts/ui-shots.sh` (light theme, 1400 px wide). They are not Basecamp window captures: in Basecamp the same view sits inside Basecamp's window. The script also renders the dark theme and two narrower widths (see [Render the screenshots](#render-the-screenshots)).

## The board

![Pukaar board with four columns of fault reports, overdue warnings and the admin's sidebar](../public/images/01-board.png)

The board for a sample site, seen by the admin, with four columns: reported, in progress, resolved and closed without fix. Cards that no steward acknowledged within 24 h, or that are past their target, are marked in red, so a stalled report is visible to everyone. The "Fix claimed, awaiting confirmation" card is still in progress, not resolved. The sidebar is the same for every member: Board, Report, History, People and Places. For the admin, People also shows a count of people waiting. Its foot shows the delivery status and the "you" block that opens Profile. A red banner appears at the top only when this device cannot rebuild the last anchored root.

## Why a report isn't closed yet

![Issue pane on a new report with the three steps to closure and the steward actions](../public/images/02-why-not-closed-yet.png)

A new report, opened by a steward. The "Why it isn't closed yet" card lists three steps: a steward acknowledges, a steward claims a fix and says what was done, then the reporter or two other residents confirm. The actions sit right under the card. Won't fix and Mark duplicate are under More, and each asks for confirmation first. A box under the buttons says why the disabled ones are not available yet. The timeline comes last.

## Confirm or reopen a fix

![Issue pane on a claimed fix, seen by the reporter, with Confirm and a separate reopen form](../public/images/03-confirm-the-fix.png)

The same pane for the reporter, on a fix a steward has claimed. A claim is not a closure: the card says who still has to act, the reporter or two other residents, and never the person who claimed the fix. "Still broken? Reopen" has its own reason box, apart from the note for comments.

## History

![History page listing resolved and closed issues by month, with filters](../public/images/04-history.png)

A table of every resolved and closed issue, with search and filters for place, status and time, grouped by month with how many were fixed within target. On a narrow window or beside the issue pane, the last four columns drop out. The board shows resolved issues for 14 days and closed ones for 30. After that they live here. Nothing is deleted, and any issue can be reopened. A status strip at the top shows the last anchor and opens Integrity.

## Report a fault

![Report form with category, description and a searchable grouped list of places](../public/images/05-report-a-fault.png)

The report form: a category, a line or two of description, and a place from the site's grouped, searchable list, or "Other" with a description. Next to the Report button, the form says what is still missing.

## Profile

![Profile page with name, identity backup, invite, my reports and leave](../public/images/06-profile.png)

The "you" block at the foot of the sidebar opens Profile: your code and role, your display name, a password-sealed backup of your identity, the site id to invite someone, a count of your reports that opens them on the Board, and "Leave this site" with a confirm step.

## People

![People page as the super admin, with one person waiting for approval, the member list with admin controls, and the removed list](../public/images/07-people.png)

Every member sees People: each member's role, with staff named and residents named or shown as "pseudonym" and their code, and "Removed" (collapsed here), which says who removed each person, when and why. Residents and stewards see it read-only. Admins can approve people and revoke members. This is the super admin's view. The site creator's key is the first super admin, and a site has at most two. Only a super admin can grant or change admins, and an admin who tries sees "Only a super admin can change admins." A super admin can transfer the role, or move an admin to steward or remove them after a notice. Reasons are asked in a confirm step, and a line under each reason box says "Everyone in the site can see this reason."

## Places and the change log

![Places page as a resident, with the places list and the change log searched for "retired"](../public/images/08-places-change-log.png)

Every member sees Places as a table, here as a resident. Each place shows its code (a link to that place in History), name, group, status and open issues, with a Changes link that searches the change log for it. The change log lists every place action (added, edited, renamed, retired, restored, removal started, removal undone and removed) with who did it, when and why, as a table, and one search field matches place, person, action or reason, here "retired". The admin also gets "Add a place" and Edit, Retire, Remove, Restore and Undo on each place, and each of these asks for a reason in a confirm step.

## Integrity

![Integrity page with the last anchor status and the three anchor steps](../public/images/09-proof.png)

The status strip at the top of History opens this page. The sidebar no longer has a Proof entry. The Board shows a red banner only on an anchor mismatch. The status block reports the newest anchor recorded in the site's history: when it was recorded and how many events it covers. It turns amber when the site was never anchored or the last anchor is older than 24 hours, and red when this device cannot rebuild the anchored root. It never says "verified", because the app does not read LEZ. Any member can anchor in three steps: compute the checkpoint, run the printed `spel` command, and record the reference it prints.

## Render the screenshots

<details>
<summary>Render the screenshots in detail</summary>

`scripts/ui-shots.sh` renders the Pukaar view offscreen against the fixture data in `scripts/ui-shots.qml`. It needs Nix at `/nix/var/nix/profiles/default/bin/nix`, and Nix gets the Qt 6.9.2 that `logos-module-builder` uses. The script renders the light and dark themes at 800, 1280 and 1400 px. It shows each state (not in a site, pending, resident, steward, admin and super admin), each page and the issue pane. It prints each QML warning. When there is a warning, the script exits with a non-zero status.

```bash
scripts/ui-shots.sh              # writes to $TMPDIR/pukaar-shots/<git short hash>
scripts/ui-shots.sh <out dir>
```

To compare two runs, you need ImageMagick. Use the `magick compare` loop in the header comment of `scripts/ui-shots.sh`.

</details>
