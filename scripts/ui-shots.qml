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
    readonly property var states: ["none", "pending", "resident", "steward", "admin"]
    readonly property var pages: ["board", "history", "report", "members", "anchor", "identity", "issue"]
    // Main.qml before named pages kept a StackLayout index; strings after.
    readonly property var pageIndex: ({ board: 0, report: 1, members: 2, anchor: 3, identity: 4, history: 5 })

    QtObject {
        id: logos
        property string state: "none"
        // Fixed within each half hour, so two runs in the same half hour give the same pixels.
        readonly property int now: Math.floor(Date.now() / 1800000) * 1800
        readonly property string site: "5173".repeat(16)
        readonly property var keys: ({ admin: "a1".repeat(32), steward: "5e".repeat(32), resident: "7e".repeat(32), r2: "72".repeat(32), pending: "9e".repeat(32) })
        readonly property var names: ({ admin: "Asha Verma", steward: "Suresh Kumar", resident: "Meera", r2: null, pending: null })
        readonly property var roles: ({ admin: "Admin", steward: "Steward", resident: "Resident", r2: "Resident" })
        function h(n) { return now - n * 3600 }
        function member(who) { return { key: keys[who], fingerprint: keys[who].substr(0, 6), name: names[who] } }
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
                  { age: 1300, acked_ts: h(1260), claimed_ts: h(1100), confirms: 1, progress: progress("steward", 1080, "New bin installed.", "", 0) })
        ]
        function ev(n, kind, who, age, body, o) {
            return Object.assign({ id: String(n).repeat(64).substr(0, 64), kind: kind, author: keys[who], author_name: names[who], ts: h(age),
                body: body, rejected: null, anchored_tx: null, anchored_by: null, anchored_by_name: null }, o || {})
        }
        function timeline(id) {
            var i = issues.filter(function (x) { return x.id === id })[0]
            if (!i) return null
            var events = [ev("e0", "report", "resident", i.reported_ts ? (now - i.reported_ts) / 3600 : 0, i.text)]
            if (i.status === "AwaitingConfirmation") events = events.concat([
                ev("e1", "acknowledge", "steward", 110, "Seen. Plumber on Thursday."),
                ev("e2", "claim_resolved", "steward", 90, "Fixed the tap."),
                ev("e3", "reopen", "resident", 85, "अब भी टपक रहा है।"),
                ev("e4", "confirm", "r2", 80, "", { rejected: "only the reporter can confirm before 48 h" }),
                ev("e5", "update", "steward", 70, "Ordered a new washer.", { anchored_tx: "ab12".repeat(16), anchored_by: keys.admin, anchored_by_name: names.admin }),
                ev("e6", "claim_resolved", "steward", 60, "Washer replaced."),
                ev("e7", "comment", "r2", 30, "Looks fine from outside.")])
            return { issue: i, events: events }
        }
        function me() {
            if (state === "none") return { site: null, delivery: "Open" }
            var who = state
            return { key: keys[who], fingerprint: keys[who].substr(0, 6), role: roles[who] || null, name: names[who], site: site,
                     syncing_own_history: false, delivery: "Open" }
        }
        function siteInfo() {
            return { site: site, name: "Ward 12, Shivaji Nagar", categories: ["Water", "Garbage", "Drainage", "Streetlight"],
                locations: locations,
                removed_locations: [{ code: "T9", label: "Old toilet block", group: "Rooms", by: keys.admin, by_name: names.admin,
                                      reason: "Demolished", since: h(1000), removed_at: h(280) }],
                members: ["admin", "steward", "resident", "r2"].map(function (w) { var m = member(w); m.role = roles[w]; return m }),
                pending: [member("pending")], sla_ack_h: 24, sla_fix_h: 72, events: 42, forks: 0, forked_authors: [] }
        }
        function reply(method, a) {
            switch (method) {
            case "my_identity": return me()
            case "site_info": return siteInfo()
            case "list_issues": return issues
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
        return out
    }
    property int job: -1
    function go(name) { var p = typeof view.item.page === "number" ? pageIndex[name] : name; view.item.openPage(p) }
    function next() {
        if (++job >= jobs.length) { Qt.quit(); return }
        var j = jobs[job], v = view.item
        if (!v || logos.state !== j.state) {
            logos.state = j.state
            view.source = ""
            view.source = "file://" + args[0]
            v = view.item
        }
        width = j.width
        v.closeIssue()
        if (j.page === "issue") { go("board"); v.openIssue(logos.issues[3].id) }
        else if (j.page !== "first-run") go(j.page)
        if (j.page === "anchor") v.checkpoint = logos.reply("checkpoint_now")
        focusSink.forceActiveFocus()
        shot.start()
    }
    Timer {
        id: shot; interval: 250
        onTriggered: {
            var j = win.jobs[win.job]
            view.grabToImage(function (r) {
                r.saveToFile(win.args[1] + "/" + j.state + "-" + j.page + "-" + j.width + ".png")
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
