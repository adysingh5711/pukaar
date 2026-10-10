// Offscreen screenshot driver for modules/pukaar_ui/Main.qml (run by ui-shots.sh).
// `logos` below stands in for the host bridge: Main.qml's unqualified `logos` resolves to it
// through the Loader's context, and it answers with fixture JSON shaped like pukaar_core's
// (logic/src/node.rs identity_json, site_info_json, issues_json, timeline_json, checkpoint_json).
// Args after "--": <Main.qml> <out dir>.
import QtQuick

Window {
    id: win
    width: 1280; height: 900; visible: true
    readonly property var args: Qt.application.arguments.slice(Qt.application.arguments.indexOf("--") + 1)
    readonly property var widths: [800, 1280, 1400]
    // "admin" is a regular admin (Kiran is the super admin); "superadmin" is Asha as the only one.
    readonly property var states: ["none", "pending", "resident", "steward", "admin", "superadmin"]
    readonly property var pages: ["board", "history", "report", "people", "places", "proof", "profile", "issue", "issue-new"]
    // The last-anchor states other than the default recent one: Integrity's block, Board's banner, History's strip.
    readonly property var anchorStates: ["never", "stale", "mismatch"]
    // Revoked roles: a removed device ("removed", as Ravi Das), and the admin's view once r2 was removed ("former").
    // Places with its change log opened and scrolled to (`log`): as the admin, then searched (a third
    // item, `search`) by a resident for an action word, and by a steward for something no row has.
    readonly property var logJobs: [["admin", "places"], ["resident", "places", "retired"], ["steward", "places", "tank lid"]]
    // Places as the admin with retired places shown and a confirm step open on a row (its objectName, the panel).
    readonly property var confirmJobs: [["placeRow-W2", "retire_location"], ["placeRow-L1", "undo_remove_location"]]
    // The last item: rows whose panel opens before the shot (People's revoke step on Meera's row).
    // Super admins: two of them; Kiran on notice, as Asha sees it and on Kiran's own Board; Kiran demoted
    // after the deadline (sealed by Suresh, one late action rejected); Asha's change and transfer steps open.
    readonly property var superJobs: [["superadmin2", "people"], ["superadmin2", "profile"], ["notice", "people"], ["noticed", "board"],
                                      ["noticed", "people"], ["demoted", "people"],
                                      ["superadmin", "people", [["memberRow-adadad", "change"], ["memberRow-a1a1a1", "transfer"]], "panels"]]
    readonly property var revokeJobs: [["removed", "board"], ["removed", "profile"], ["admin", "board", true], ["admin", "issue", true],
                                       ["admin", "people", true, [["memberRow-7e7e7e", "revoke"]]]]

    QtObject {
        id: logos
        property string state: "none"
        property string anchor: "recent"     // site_info last_anchor: recent, never, stale or mismatch
        property bool former: false          // r2 was removed: their reports and timeline entries say so
        // Fixed within each half hour, so two runs in the same half hour give the same pixels.
        readonly property int now: Math.floor(Date.now() / 1800000) * 1800
        readonly property string site: "5173".repeat(16)
        readonly property var keys: ({ admin: "a1".repeat(32), admin2: "ad".repeat(32), steward: "5e".repeat(32), resident: "7e".repeat(32), r2: "72".repeat(32), pending: "9e".repeat(32), gone: "6a".repeat(32) })
        readonly property var names: ({ admin: "Asha Verma", admin2: "Kiran Shah", steward: "Suresh Kumar", resident: "Meera", r2: null, pending: null, gone: "Ravi Das" })
        readonly property var roles: ({ admin: "Admin", admin2: state === "demoted" ? "Steward" : "Admin", steward: "Steward", resident: "Resident", r2: "Resident" })
        function h(n) { return now - n * 3600 }
        // Who is looking (most states are a role's own key), and who the super admins are.
        readonly property string viewer: ({ removed: "gone", noticed: "admin2", superadmin: "admin", superadmin2: "admin", notice: "admin", demoted: "admin" })[state] || state
        readonly property var supers: state === "admin" ? ["admin2"] : state === "superadmin2" ? ["admin", "admin2"] : ["admin"]
        // Asha's notice to Kiran: running (a week to go), or ended and sealed by Suresh (`demoted`).
        function notice(sealed) {
            return { subject: keys.admin2, subject_name: names.admin2, by: keys.admin, by_name: names.admin, ts: h(20), outcome: "steward",
                     deadline: sealed ? h(10) : now + 7 * 86400, reason: "Missed two audits, and shared a key with a helper.", sealed: sealed,
                     sealed_by: sealed ? keys.steward : null, sealed_by_name: sealed ? names.steward : null, sealed_ts: sealed ? h(9) : null }
        }
        readonly property var notices: state === "notice" || state === "noticed" ? [notice(false)] : state === "demoted" ? [notice(true)] : []
        function member(who) {
            return { key: keys[who], fingerprint: keys[who].substr(0, 6), name: names[who], super_admin: supers.indexOf(who) >= 0,
                     demotions: state === "demoted" && who === "admin2" ? [notice(true)] : [] }
        }
        function revocation(age, reason) { return { by: keys.admin, by_name: names.admin, ts: h(age), reason: reason } }
        readonly property var revocations: ({ gone: revocation(30, "Left the camp."), r2: revocation(5, "Shared the site id outside the ward.") })
        function revoked() {
            return (former ? ["r2", "gone"] : ["gone"]).map(function (w) { return Object.assign(member(w), revocations[w]) })
        }
        function marked(i) { return Object.assign({}, i, { former: former && i.reporter === keys.r2 }) }
        function loc(code, label, group, extra) {
            return Object.assign({ code: code, label: label, group: group, state: "active", retired: false, retired_reason: "",
                removal_reason: "", removes_at: 0, renamed_from: null, ever_used: true, open_issues: 0 }, extra || {})
        }
        readonly property var locations: [
            loc("W1", "हैंडपंप, मंदिर के पास", "Water points", { open_issues: 2 }),
            loc("W2", "Tank by the school", "Water points", { renamed_from: "Old school tank" }),
            loc("B1", "कूड़ादान, बाज़ार गेट", "Bins", { open_issues: 1 }),
            loc("P1", "गली नंबर 4 की नाली", "Paths", { open_issues: 1 }),
            loc("P2", "Lane behind the clinic", "Paths", { state: "retired", retired: true, retired_reason: "Lane closed for construction" }),
            loc("L1", "Streetlight pole 17", "Paths", { state: "pending_removal", removal_reason: "Duplicate of P1", removes_at: now + 12 * 86400, ever_used: false })
        ]
        // site_info place_log: every kind, newest first, one over the 20 shown before "+N older".
        function pc(kind, code, age, reason, extra) {
            var l = locations.filter(function (x) { return x.code === code })[0]
            return Object.assign({ ts: h(age), by: keys.admin, by_name: names.admin, code: code,
                                   label: l ? l.label : "Old toilet block", kind: kind, reason: reason }, extra || {})
        }
        readonly property var placeLog: [
            pc("retired", "P2", 150, "Lane closed for construction"),
            pc("removed", "T9", 280, "Demolished", { since: h(1000) }),
            pc("removal_started", "L1", 432, "Duplicate of P1: the same pole was entered twice when the ward map was first set up, so this copy goes.",
               { ends_at: now + 12 * 86400 }),
            pc("renamed", "W2", 500, "", { from: "Old school tank", to: "Tank by the school" }),
            pc("removal_undone", "L1", 650, "The pole is staying after all."),
            pc("removal_started", "L1", 700, "Not needed", { ends_at: h(700) + 720 * 3600 }),
            pc("restored", "P2", 900, "Lane reopened"),
            pc("retired", "P2", 950, "Waterlogged"),
            pc("removal_started", "T9", 1000, "Demolished", { ends_at: h(280) }),
            pc("renamed", "P1", 1020, "", { from: "Drain, lane 4", to: "गली नंबर 4 की नाली" }),
            pc("edited", "P1", 1050, ""),
            pc("edited", "B1", 1100, ""),
            pc("restored", "W1", 1150, "Pump repaired"),
            pc("retired", "W1", 1200, "Pump dry"),
        ].concat(["W1", "W2", "B1", "P1", "P2", "L1", "T9"].map(function (c, n) { return pc("added", c, 1400 + n, "") }))
        function progress(by, ts, note, next, eta) {
            return { by: keys[by], by_name: names[by], ts: h(ts), note: note, next_step: next, eta_h: eta, due_ts: eta ? h(ts) + eta * 3600 : 0 }
        }
        function issue(n, status, stage, code, cat, text, rep, o) {
            var l = locations.filter(function (x) { return x.code === code })[0]
            return Object.assign({ id: String(n).repeat(64).substr(0, 64), status: status, stage: stage, category: cat, location: code,
                location_label: l ? l.label : null, location_renamed_from: l ? l.renamed_from : null, location_retired: !!(l && l.retired),
                landmark: "", text: text, progress: null, reporter: keys[rep], reporter_name: names[rep], claimant: null, confirms: 0,
                reopen_count: 0, reported_ts: h(o.age), acked_ts: null, claimed_ts: null, ack_overdue: false, fix_overdue: false, awaiting_48h: false }, o)
        }
        readonly property var issues: [
            issue(1, "Open", "Reported", "W1", "Water", "हैंडपंप से पानी नहीं आ रहा, दो दिन से बंद है।", "resident", { age: 30, ack_overdue: true }),
            issue(2, "Acknowledged", "Reported", "B1", "Garbage", "Bin overflowing onto the road since Monday.", "r2",
                  { age: 60, acked_ts: h(50), fix_overdue: true, progress: progress("steward", 50, "Seen, truck booked.", "", 24) }),
            issue(3, "InProgress", "In progress", "P1", "Drainage", "नाली जाम है, बारिश में पानी घरों में घुसता है।", "resident",
                  { age: 20, acked_ts: h(18), progress: progress("steward", 6, "Crew cleared half the drain.", "Finish the rest tomorrow", 48) }),
            issue(4, "AwaitingConfirmation", "Fix claimed, awaiting confirmation", "W1", "Water", "Tap at the temple pump leaks all night.", "resident",
                  { age: 120, acked_ts: h(110), claimed_ts: h(60), claimant: keys.steward, awaiting_48h: true, reopen_count: 1,
                    progress: progress("steward", 60, "Washer replaced.", "", 0) }),
            issue(5, "ConfirmedResolved", "Resolved", "W2", "Water", "Tank lid missing.", "r2",
                  { age: 100, acked_ts: h(98), claimed_ts: h(70), confirms: 1, progress: progress("steward", 48, "Confirmed by reporter.", "", 0) }),
            issue(6, "ClosedWontfix", "Closed: won't fix", "P2", "Streetlight", "Light out behind the clinic.", "resident",
                  { age: 200, acked_ts: h(190), progress: progress("admin", 100, "Lane is closed for construction.", "", 0) }),
            issue(7, "Duplicate", "Closed: duplicate", "other", "Garbage", "Bin at the market gate is full.", "r2",
                  { age: 58, landmark: "बाज़ार गेट के सामने", progress: progress("steward", 55, "Same as the B1 report.", "", 0) }),
            issue(8, "ConfirmedResolved", "Resolved", "B1", "Garbage", "Old bin broken.", "resident",
                  { age: 1300, acked_ts: h(1260), claimed_ts: h(1100), confirms: 1, reopen_count: 1, progress: progress("steward", 1080, "New bin installed.", "", 0) })
        ]
        function ev(n, kind, who, age, body, o) {
            return Object.assign({ id: String(n).repeat(64).substr(0, 64), kind: kind, author: keys[who], author_name: names[who], ts: h(age),
                body: body, rejected: null, anchored_tx: null, anchored_by: null, anchored_by_name: null, former: former && who === "r2" }, o || {})
        }
        function timeline(id) {
            var i = issues.filter(function (x) { return x.id === id })[0]
            if (!i) return null
            var rep = Object.keys(keys).filter(function (w) { return keys[w] === i.reporter })[0]
            var events = [ev("e0", "report", rep, i.reported_ts ? (now - i.reported_ts) / 3600 : 0, i.text)]
            if (i.status === "AwaitingConfirmation") events = events.concat([
                ev("e1", "acknowledge", "steward", 110, "Seen. Plumber on Thursday."),
                ev("e2", "claim_resolved", "steward", 90, "Fixed the tap."),
                ev("e3", "reopen", "resident", 85, "अब भी टपक रहा है।"),
                ev("e4", "confirm", "r2", 80, "", { rejected: "only the reporter can confirm before 48 h" }),
                ev("e5", "update", "steward", 70, "Ordered a new washer.", { anchored_tx: "ab12".repeat(16), anchored_by: keys.admin, anchored_by_name: names.admin }),
                ev("e6", "claim_resolved", "steward", 60, "Washer replaced."),
                ev("e7", "comment", "r2", 30, "Looks fine from outside.")])
            return { issue: marked(i), events: events }
        }
        function me() {
            if (state === "none") return { site: null, delivery: "Open" }
            var who = viewer
            return { key: keys[who], fingerprint: keys[who].substr(0, 6), role: roles[who] || null, name: names[who], site: site,
                     super_admin: supers.indexOf(who) >= 0, notice: state === "noticed" ? notice(false) : null,
                     revoked: revocations[who] || null, syncing_own_history: false, delivery: "Open" }
        }
        function siteInfo() {
            return { site: site, name: "Ward 12, Shivaji Nagar", categories: ["Water", "Garbage", "Drainage", "Streetlight"],
                locations: locations,
                removed_locations: [{ code: "T9", label: "Old toilet block", group: "Rooms", by: keys.admin, by_name: names.admin,
                                      reason: "Demolished", since: h(1000), removed_at: h(280) }],
                place_log: placeLog,
                members: ["admin", "admin2", "steward", "resident"].concat(former ? [] : ["r2"]).map(function (w) { var m = member(w); m.role = roles[w]; return m }),
                pending: [member("pending")], revoked: revoked(), super_admins: supers.map(member), notices: notices,
                rejected: state === "demoted" ? [{ id: "e9".repeat(32), kind: "role_grant", author: keys.admin2, author_name: names.admin2,
                                                   ts: h(8), reason: "only admin grants roles" }] : [], sla_ack_h: 24, sla_fix_h: 72, events: 42, forks: 0, forked_authors: [],
                last_anchor: anchor === "never" ? null : { ts: h(anchor === "stale" ? 50 : 3), tx: "pda:Public/9f2c", by: keys.steward,
                    by_name: names.steward, events_covered: 33, reproducible: anchor !== "mismatch" } }
        }
        function reply(method, a) {
            switch (method) {
            case "my_identity": return me()
            case "site_info": return siteInfo()
            case "list_issues": return issues.map(marked)
            case "issue_timeline": return timeline(a[0])
            case "checkpoint_now": return { heads: [[keys.admin, 12]], heads_root: "c0ffee".repeat(10) + "abcd", n_events: 42, signer: keys.admin,
                sig: "5a".repeat(64), spel: "spel anchor --site-id " + site + " --heads-root … --payer <YOUR_PUBLIC_ACCOUNT>" }
            }
            return "error: the screenshot stub does not do " + method
        }
        function callModule(module, method, a) { return JSON.stringify(reply(method, a)) }
    }

    Loader { id: view; anchors.fill: parent }
    Item { id: focusSink }   // takes focus before each shot: a blinking text cursor would differ between runs

    property var jobs: {
        var out = []
        states.forEach(function (s) {
            widths.forEach(function (w) {
                (s === "none" ? ["first-run"] : pages).forEach(function (p) { out.push({ state: s, width: w, page: p }) })
            })
        })
        superJobs.forEach(function (r) {
            [800, 1400].forEach(function (w) { out.push({ state: r[0], width: w, page: r[1], panels: r[2], tag: r[3] }) })
        })
        revokeJobs.forEach(function (r) {
            [800, 1400].forEach(function (w) { out.push({ state: r[0], width: w, page: r[1], former: !!r[2], panels: r[3] }) })
        })
        anchorStates.forEach(function (a) {
            [800, 1400].forEach(function (w) { ["proof", "board", "history"].forEach(function (p) { out.push({ state: "resident", width: w, page: p, anchor: a }) }) })
        })
        logJobs.forEach(function (r) {
            [800, 1400].forEach(function (w) { out.push({ state: r[0], width: w, page: r[1], log: true, search: r[2] || "" }) })
        })
        ;[800, 1400].forEach(function (w) { out.push({ state: "admin", width: w, page: "places", rows: true, panels: confirmJobs }) })
        return out
    }
    property int job: -1
    function find(item, name) {
        if (item.objectName === name) return item
        for (var i = 0; i < item.children.length; i++) { var f = find(item.children[i], name); if (f) return f }
        return null
    }
    function next() {
        if (++job >= jobs.length) { Qt.quit(); return }
        var j = jobs[job], v = view.item
        var anchor = j.anchor || "recent"
        if (!v || logos.state !== j.state || logos.anchor !== anchor || logos.former !== !!j.former) {
            logos.state = j.state
            logos.anchor = anchor
            logos.former = !!j.former
            view.source = ""
            view.source = "file://" + args[0]
            v = view.item
            v.now = logos.now   // its clock too: a notice's deadline shows to the minute
        }
        width = j.width
        v.closeIssue()
        var pane = ["issue", "issue-new"].indexOf(j.page)   // the pane on a claimed fix (a removed member's report with `former`), and on a new report (More)
        if (pane >= 0) { v.openPage("board"); v.openIssue(logos.issues[pane ? 0 : j.former ? 1 : 3].id) }
        else if (j.page !== "first-run") v.openPage(j.page)
        if (j.page === "proof") v.checkpoint = logos.reply("checkpoint_now")
        var log = find(v, "changeLog")
        if (log) log.open = !!j.log
        var search = find(v, "logSearch")
        if (search) search.text = j.search || ""
        var rows = find(v, "placesPage")
        if (rows) find(rows, "showRetired").checked = !!j.rows
        var members = find(v, "memberList")
        if (members) members.open = j.page === "people"   // an admin's opens collapsed: show its rows
        focusSink.forceActiveFocus()
        shot.start()
    }
    Timer {
        id: shot; interval: 250
        onTriggered: {
            var j = win.jobs[win.job]
            if (j.panels && !j.armed) {   // the rows exist now: open their panels, then wait for the layout
                j.armed = true
                j.panels.forEach(function (c) { win.find(view.item, c[0]).panel = c[1] })
                return shot.restart()
            }
            if (j.rows) {   // scrolled to the end of the page: the last row's confirm step
                var page = win.find(view.item, "placesPage").contentItem
                page.contentY = page.contentHeight - page.height
            }
            if (j.log) {   // scroll the page so the change log's header is at the top
                var log = win.find(view.item, "changeLog"), f = log.parent
                while (f.contentY === undefined) f = f.parent
                f.contentY = Math.min(log.mapToItem(f.contentItem, 0, 0).y - 16, f.contentHeight - f.height)
            }
            focusSink.forceActiveFocus()   // again: an opened panel puts the cursor in its field
            view.grabToImage(function (r) {
                r.saveToFile(win.args[1] + "/" + j.state + "-" + j.page + (j.anchor ? "-anchor-" + j.anchor : "") + (j.former ? "-former" : "") + (j.log ? "-log" : "") + (j.rows ? "-rows" : "") + (j.search ? "-search" : "") + (j.tag ? "-" + j.tag : "") + "-" + j.width + ".png")
                win.next()
            })
        }
    }
    Component.onCompleted: {
        // Main.qml's Settings (the sidebar toggle) go to a store of their own, never toggled here: the
        // default width rule applies, whatever the qml runtime remembered from another session.
        Qt.application.organization = Qt.application.name = "pukaar-ui-shots"
        next()
    }
}
