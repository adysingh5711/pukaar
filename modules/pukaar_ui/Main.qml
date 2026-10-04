// Pukaar view. QML-only: every call goes to pukaar_core through the host's
// `logos` bridge, and the view re-reads state every 2 s (no module events at L1).
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Shapes

Item {
    id: root
    property var me: ({})
    readonly property var noInfo: ({ categories: [], locations: [], removed_locations: [], members: [], pending: [] })
    property var info: noInfo
    // The lists of site_info live in properties of their own, written only when their content
    // changed: info itself changes on every received event (it carries an event count), and a
    // rebuilt model would wipe typed text in the Members rows and reset the Report combos.
    property var categories: []
    property var locations: []
    property var removedLocations: []    // the admin's change log: places whose 30-day removal is over
    property var members: []
    property var pending: []
    property var groupNames: []             // place groups in first-seen order; written only when they change
    property var issues: []
    property var selected: null          // issue_timeline() result
    property var checkpoint: null        // checkpoint_now() result
    property string message: ""
    property bool onlyMine: false
    property bool busy: false            // a call is running: every action button waits (no double submit)
    readonly property bool inSite: !!(me && me.site)
    readonly property bool staff: me.role === "Steward" || me.role === "Admin"
    readonly property bool approved: !!me.role      // a pending key has no role yet; the core rejects its reports
    readonly property bool isAdmin: me.role === "Admin"
    // A restored identity waits for its own history before signing anything (core refuses meanwhile).
    readonly property bool syncing: !!me.syncing_own_history
    // Four board columns (160 each) + the 340 timeline pane + gaps and margins need ~1040 px;
    // below that the pane replaces the board instead of squeezing it.
    readonly property bool narrow: width < 1040

    // ---- design tokens: the only place a colour or the stage->colour map is spelled out ----
    // Two sets (design/proposal/NOTES.md section 1): light = Primer, dark = Logos neutrals with
    // Primer's stage hues. Everything reads root.t, which follows the system colour scheme live.
    // Names match the mockup's CSS variables; the sb*/tip* roles are for the sidebar (step 5).
    // QML spells alpha first (#AARRGGBB): CSS #54aeff99 is "#9954aeff" here.
    readonly property bool dark: Application.styleHints.colorScheme === Qt.Dark
    readonly property var lightTokens: ({
        fg: "#1f2328", muted: "#59636e", ph: "#6e7781", disabled: "#818b98",
        page: "#ffffff", well: "#f6f8fa", card: "#ffffff", field: "#ffffff", hover: "#eff2f5", btn: "#f6f8fa", btnOff: "#f6f8fa",
        bd: "#d1d9e0", ctlBd: "#818b98", btnBd: "#d1d9e0",
        accent: "#0969da", accentBg: "#ddf4ff", accentBd: "#9954aeff", focus: "#0969da",
        primary: "#0969da", primaryHover: "#0860ca", onPrimary: "#ffffff",
        danger: "#d1242f", dangerBg: "#ffebe9", dangerBd: "#99ff8182",
        okDot: "#1a7f37", connDot: "#9a6700",
        reported: { fg: "#bc4c00", bg: "#fff1e5", bd: "#99fb8f44", edge: "#bc4c00" },
        progress: { fg: "#0969da", bg: "#ddf4ff", bd: "#9954aeff", edge: "#0969da" },
        awaiting: { fg: "#8250df", bg: "#fbefff", bd: "#99c297ff", edge: "#8250df" },
        resolved: { fg: "#1a7f37", bg: "#dafbe1", bd: "#994ac26b", edge: "#1a7f37" },
        closed: { fg: "#59636e", bg: "#eff1f3", bd: "#d1d9e0", edge: "#818b98" },
        sb: "#f6f8fa", sbCard: "#ffffff", sbHover: "#eaeef2", sbFg: "#1f2328", sbMuted: "#59636e", sbIcon: "#59636e",
        sbEdge: "#d1d9e0", sbLine: "#d1d9e0", sbCtl: "#818b98", sbActBg: "#c8e6ff", sbActFg: "#0550ae", sbActIcon: "#0550ae",
        sbDisabled: "#59636e", sbOk: "#1a7f37", sbConn: "#9a6700", sbOff: "#d1242f", sbDanger: "#d1242f",
        tipBg: "#25292e", tipFg: "#ffffff", tipBd: "#25292e"
    })
    readonly property var darkTokens: ({
        fg: "#ffffff", muted: "#a4a4a4", ph: "#8c92a0", disabled: "#7a7a7a",
        page: "#171717", well: "#1c1c1c", card: "#232323", field: "#262626", hover: "#2b2b2b", btn: "#2b2b2b", btnOff: "#1f1f1f",
        bd: "#434343", ctlBd: "#808080", btnBd: "#4d4d4d",
        accent: "#6aa8f0", accentBg: "#1f2d3d", accentBd: "#4a90e2", focus: "#6aa8f0",
        primary: "#2563c9", primaryHover: "#2f6fd0", onPrimary: "#ffffff",
        danger: "#ff8a82", dangerBg: "#2e1f1f", dangerBd: "#fb3748",
        okDot: "#49f563", connDot: "#febc2e",
        reported: { fg: "#ff9a2e", bg: "#3b2814", bd: "#ff8800", edge: "#ff8800" },
        progress: { fg: "#6aa8f0", bg: "#1f2d3d", bd: "#4a90e2", edge: "#4a90e2" },
        awaiting: { fg: "#b794f6", bg: "#2f2840", bd: "#a07bea", edge: "#a07bea" },
        resolved: { fg: "#6ccc93", bg: "#22362b", bd: "#6ccc93", edge: "#6ccc93" },
        closed: { fg: "#b5b5b5", bg: "#2f2f2f", bd: "#808080", edge: "#808080" },
        sb: "#141414", sbCard: "#1c1c1c", sbHover: "#262626", sbFg: "#ebebeb", sbMuted: "#a4a4a4", sbIcon: "#a4a4a4",
        sbEdge: "#2c2c2c", sbLine: "#343434", sbCtl: "#808080", sbActBg: "#243b55", sbActFg: "#ffffff", sbActIcon: "#6aa8f0",
        sbDisabled: "#8a8a8a", sbOk: "#49f563", sbConn: "#febc2e", sbOff: "#ff736a", sbDanger: "#ff8a82",
        tipBg: "#3a3a3a", tipFg: "#ffffff", tipBd: "#808080"
    })
    readonly property var t: dark ? darkTokens : lightTokens
    // Every control reads these roles. Set once on the root, so nothing falls back to the
    // system palette (a dark OS would otherwise give light-on-light or dark-on-dark controls).
    // Same value in the disabled and inactive groups: disabled controls use the `disabled` /
    // `btnOff` tokens (ink() below), never opacity, so their contrast is checked like the rest.
    palette {
        window: root.t.page; windowText: root.t.fg
        base: root.t.field; alternateBase: root.t.well; text: root.t.fg
        button: root.t.btn; buttonText: root.t.fg; brightText: root.t.onPrimary
        placeholderText: root.t.ph
        highlight: root.t.primary; highlightedText: root.t.onPrimary
        link: root.t.accent; linkVisited: root.t.accent
        toolTipBase: root.t.tipBg; toolTipText: root.t.tipFg
        light: root.t.page; midlight: root.t.hover; mid: root.t.ctlBd
        dark: root.t.muted; shadow: root.t.fg
    }
    // Sizes follow the user's system font instead of fixed pixels.
    FontMetrics { id: systemFont }       // default font = the application's
    readonly property real baseSize: systemFont.font.pointSize > 0 ? systemFont.font.pointSize : 10
    readonly property real smallSize: baseSize * 13 / 14      // meta text: 13 px when the body is 14
    readonly property real titleSize: baseSize * 1.6
    // Stage -> {fg, bg, bd, edge}: text, tint, border, strip. The only orange is Reported.
    readonly property var stageGroup: ({
        Open: "reported", Acknowledged: "reported", InProgress: "progress", AwaitingConfirmation: "awaiting",
        ConfirmedResolved: "resolved", ClosedWontfix: "closed", Duplicate: "closed"
    })
    readonly property var groupIcon: ({ reported: "rep", progress: "prog", awaiting: "await", resolved: "res", closed: "clo" })
    function stageIcon(status) { return groupIcon[stageGroup[status]] || "warn" }
    function stageStyle(status) {
        return t[stageGroup[status]] || { fg: t.danger, bg: t.dangerBg, bd: t.dangerBd, edge: t.danger }
    }
    // Statuses a steward can still act on (acknowledge / update / claim / won't-fix).
    function isActionable(status) { return ["Open", "Acknowledged", "InProgress"].indexOf(status) >= 0 }

    // Raw reply of each polled method as last applied. Not bound to anything: poll() compares
    // against it and skips the assignment when nothing changed, so a quiet 2 s tick resets no model.
    property var seen: ({})
    property bool polling: false         // one poll in flight at a time
    property bool repoll: false          // an action finished mid-poll: poll again right after

    // This is the ONE place that talks to pukaar_core: every button routes through run()/call()/poll().
    // Async (`callModuleAsync`) so a slow core call (Delivery IPC, big replay) never freezes the UI;
    // `done(raw)` runs once with the reply string. A host without it falls back to the blocking call.
    function send(method, args, done) {
        // qmllint disable unqualified
        // `logos` is injected by the C++ host at runtime; it has no static QML type.
        if (typeof logos === "undefined" || !logos.callModule)
            done("error: no logos bridge (run inside Basecamp or logos-standalone-app)")
        else if (logos.callModuleAsync)
            logos.callModuleAsync("pukaar_core", method, args, done)
        else
            done(logos.callModule("pukaar_core", method, args))
        // qmllint enable unqualified
    }
    // Replies arrive as JSON strings, sometimes double-encoded by the bridge; the bridge's own
    // failures ({"error": …}) are folded into the "error: …" strings the core uses.
    function decode(v) {
        try { v = JSON.parse(v) } catch (e) {}
        if (typeof v === "string") { try { v = JSON.parse(v) } catch (e) {} }
        if (v && typeof v.error === "string") return "error: " + v.error + (v.message ? ": " + v.message : "")
        return v
    }
    function call(method, args, done) { send(method, args, function (raw) { done(decode(raw)) }) }
    // Read-only refresh: hand the decoded reply to `apply` only if it differs from the last one.
    function poll(method, args, apply, next) {
        send(method, args, function (raw) {
            if (raw !== seen[method]) { seen[method] = raw; apply(decode(raw)) }
            if (next) next()
        })
    }
    function keep(name, value) { if (JSON.stringify(value) !== JSON.stringify(root[name])) root[name] = value }
    function setInfo(i) {
        info = i
        keep("categories", i.categories); keep("locations", i.locations); keep("removedLocations", i.removed_locations || [])
        keep("members", i.members); keep("pending", i.pending)
        keep("groupNames", groupsOf(i.locations))
    }
    // Opening a card moves the cursor to the note, so a steward can type straight away.
    function openIssue(id) {
        call("issue_timeline", [id], function (t) {
            if (!t || !t.issue) return
            selected = t
            note.forceActiveFocus()
        })
    }
    function closeIssue() { selected = null }
    // The core's Delivery state, in plain words (the raw Down/Listening/Starting/Started/Open are internal).
    function deliveryLabel(d) {
        if (failed(d)) return "offline: " + d.substr("error: ".length)
        return d === "Open" ? "online" : "connecting"
    }
    function failed(v) {
        return typeof v === "string" && (v.indexOf("error:") === 0 || v.indexOf("rejected:") === 0)
    }
    // Every action goes through here. `onOk(reply)` runs only when the core accepted it, so
    // typed text survives a refusal and the user can fix it and retry. `hint`, if given, is
    // added to a refusal that is about size ("too long"), saying what to do instead.
    function run(method, args, onOk, hint) {
        if (busy) return
        busy = true
        call(method, args, function (r) {
            busy = false
            message = !failed(r) ? "" : hint && r.indexOf("too long") >= 0 ? r + ". " + hint : r
            refresh()
            if (!failed(r) && onOk) onOk(r)
        })
    }
    // Drop this device's copy of the site (the key stays); back to the first-run screen.
    function leave() {
        run("leave_site", [], function () {
            closeIssue(); checkpoint = null; issues = []; setInfo(noInfo); seen = ({})
        })
    }
    // Poll steps in order; the site-bound ones only once my_identity says we are in a site.
    function pollSteps(steps, i) {
        if (i >= steps.length || (i > 0 && !inSite)) {
            polling = false
            if (repoll) { repoll = false; refresh() }
            return
        }
        poll(steps[i][0], steps[i][1], steps[i][2], function () { pollSteps(steps, i + 1) })
    }
    function refresh() {
        if (polling) { repoll = true; return }
        polling = true
        var steps = [
            ["my_identity", [], function (m) { me = (m && typeof m === "object") ? m : {} }],
            ["site_info", [], function (i) { if (i && i.site) setInfo(i) }],
            ["list_issues", [], function (l) { if (Array.isArray(l)) issues = l }]
        ]
        // only refresh the pane that is still open (a reply landing after Close must not reopen it)
        if (selected) {
            var open = selected.issue.id
            steps.push(["issue_timeline", [open], function (t) {
                if (t && t.issue && selected && selected.issue.id === t.issue.id) selected = t
            }])
        }
        pollSteps(steps, 0)
    }
    function inColumn(statuses) {
        return issues.filter(function (i) {
            return statuses.indexOf(i.status) >= 0 && (!onlyMine || i.reporter === me.key)
        })
    }
    readonly property int boardCount: issues.filter(function (i) { return !onlyMine || i.reporter === me.key }).length
    // Clipboard via a hidden TextEdit (QML has no direct clipboard API); header Copy and FramedTextArea share it.
    TextEdit { id: clip; visible: false }
    function copyText(t) { clip.text = t; clip.selectAll(); clip.copy() }
    function shortId(hex) { return String(hex).substr(0, 8) + "…" + String(hex).substr(-4) }
    function who(key, name) { return name ? name : "pseudonym " + String(key).substr(0, 6) }
    function when(ts) { return new Date(ts * 1000).toLocaleString(Qt.locale(), "d MMM HH:mm") }
    function esc(s) { return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;") }
    function place(i) {
        var p = i.location === "other" ? "Other" : (i.location + " " + (i.location_label || ""))
        if (i.location_retired) p += " (retired)"
        return i.landmark ? p + " (" + i.landmark + ")" : p
    }
    function placeCode(i) { return i.location === "other" ? "Other" : i.location }
    // place() without its code, which the card sets apart; "Other" shows its landmark bare.
    function placeName(i) { return i.location === "other" ? (i.landmark || "") : place(i).substr(i.location.length + 1) }
    function plural(n, word) { return n + " " + word + (n === 1 ? "" : "s") }
    // A place that takes no reports, in words: "retired" or "removes in 12 days" ("" = active).
    function placeState(l) {
        if (l.state === "pending_removal")
            return "removes in " + plural(Math.max(0, Math.ceil((l.removes_at - Date.now() / 1000) / 86400)), "day")
        return l.state === "retired" ? "retired" : ""
    }
    function dueHours(p) { return Math.round((p.due_ts - Date.now() / 1000) / 3600) }
    function isOverdue(i) {
        var p = i.progress
        return !!p && p.due_ts > 0 && isActionable(i.status) && dueHours(p) < 0
    }
    function age(ts) {
        var h = Math.floor((Date.now() / 1000 - ts) / 3600)
        return h < 1 ? "just now" : h < 48 ? h + " h ago" : Math.floor(h / 24) + " d ago"
    }
    // The card's red lines. SLA flags are computed by the core (site SLA hours); this only words them.
    function flagLines(i) {
        var out = []
        if (i.ack_overdue) out.push("Not acknowledged within " + info.sla_ack_h + " h.")
        else if (i.fix_overdue) out.push("Past the " + info.sla_fix_h + " h fix target.")
        else if (i.awaiting_48h) out.push("Fix unconfirmed for over 48 h.")
        if (isOverdue(i)) out.push("OVERDUE by " + -dueHours(i.progress) + " h")
        return out
    }
    // ---- who may act on an issue: the core's rules (logic/src/reducer.rs), each as the reason the
    // viewer can't, checked in the core's order; "" = allowed. The pane shows an action when its
    // rule says "", and unavailable() lists the others with these reasons. ----
    function stageWhy(st) {
        return isActionable(st) ? "no fix has been claimed yet" : st === "AwaitingConfirmation" ? "a fix is already claimed"
             : st === "ConfirmedResolved" ? "it is resolved" : "it is closed"
    }
    // The reporter decides alone; any other resident is one of the two votes (confirm and reopen alike).
    function canVote(i) { return i.reporter === me.key || me.role === "Resident" }
    // Post update, Claim fixed, Won't fix, Mark duplicate (and Acknowledge while Open).
    function stewardWhy(i) { return !staff ? "only stewards can" : isActionable(i.status) ? "" : stageWhy(i.status) }
    function confirmWhy(i) {
        return i.status !== "AwaitingConfirmation" ? stageWhy(i.status)
             : i.claimant === me.key ? "you claimed this fix"
             : canVote(i) ? "" : "only the reporter or a resident can"
    }
    function reopenWhy(i) { return isActionable(i.status) ? stageWhy(i.status) : canVote(i) ? "" : "only the reporter or a resident can" }
    function needsMe(i) { return i.reporter === me.key && confirmWhy(i) === "" }
    // "Not available to you": every action the pane hides or disables, one line per reason, in the
    // mockup's words ("Reopen and Comment need a note first."). The core refuses these without a note.
    function unavailable(i, hasNote) {
        var note = hasNote ? "" : "note", steward = stewardWhy(i)
        var why = [["Acknowledge", staff ? "" : steward]]
        ;["Post update", "Claim fixed", "Won't fix"].forEach(function (a) { why.push([a, steward || note]) })
        why.push(["Confirm it's fixed", confirmWhy(i)], ["Reopen", reopenWhy(i) || note], ["Comment", note])
        var reasons = [], acts = {}
        why.forEach(function (w) {
            if (!w[1]) return
            if (!acts[w[1]]) { acts[w[1]] = []; reasons.push(w[1]) }
            acts[w[1]].push(w[0])
        })
        return reasons.map(function (r) {
            var a = acts[r], names = a.length > 1 ? a.slice(0, -1).join(", ") + " and " + a[a.length - 1] : a[0]
            return r === "note" ? names + (a.length > 1 ? " need" : " needs") + " a note first." : names + ", because " + r + "."
        })
    }
    // "Why it isn't closed yet": what still has to happen, by the same rules (the reporter confirms,
    // or 2 other residents do; the fixer never confirms their own fix). null once it is closed.
    function whyCard(i) {
        if (i.status === "AwaitingConfirmation") {
            var p = i.progress || {}
            var fixer = i.claimant === me.key ? "you" : esc(who(i.claimant, p.by === i.claimant ? p.by_name : ""))
            var mine = i.reporter === me.key
            return {
                intro: "Fix claimed by <b>" + fixer + "</b>" + (p.ts ? " on " + when(p.ts) : "") + ". A claim is not a closure.",
                steps: [
                    { title: mine ? "<b>You (the reporter) confirm</b>" : "<b>" + esc(who(i.reporter, i.reporter_name)) + "</b> (the reporter) confirms",
                      hint: i.claimant === i.reporter ? "Not possible here: the reporter claimed this fix."
                          : mine ? "Or reopen it, with a reason, if it still isn't fixed." : "Waiting. The reporter can also reopen at any time." },
                    { title: "<b>or 2 other residents confirm</b>", meter: i.confirms, hint: i.confirms + " of 2 so far" }
                ],
                never: fixer === "you" ? "<b>You claimed this fix, so you can't confirm it.</b>" : fixer + " claimed this fix, so " + fixer + " can't confirm it."
            }
        }
        if (!isActionable(i.status)) return null
        var late = Math.floor((Date.now() / 1000 - i.reported_ts) / 3600) - info.sla_ack_h
        return { intro: "", never: "", steps: [
            { title: "<b>A steward acknowledges it</b>", done: i.status !== "Open",
              hint: i.status !== "Open" ? "Done." : "Not done yet." + (i.ack_overdue ? " " + late + " h past the " + info.sla_ack_h + " h target." : "") },
            { title: "<b>A steward claims a fix and says what was done</b>", hint: i.fix_overdue ? "Not done yet. Past the " + info.sla_fix_h + " h fix target." : "" },
            { title: "<b>The reporter, or 2 other residents, confirm it</b>", hint: "" }
        ] }
    }
    // Timeline kinds in words: [label, noun when rejected, icon, label when a resident other than the
    // reporter did it (one of two votes, so it may not have changed the status)]. The raw kind shows beside it.
    readonly property var kinds: ({
        report: ["Reported", "Report", "rep"], acknowledge: ["Acknowledged", "Acknowledgement", "eye"],
        update: ["Update posted", "Update", "update"], claim_resolved: ["Fix claimed", "Fix claim", "flag"],
        confirm: ["Confirmed fixed", "Confirmation", "res", "Resident confirmed"],
        reopen: ["Reopened", "Reopen", "reopen", "Resident asked to reopen"],
        close_wontfix: ["Closed: won't fix", "Won't fix", "clo"], mark_duplicate: ["Marked duplicate", "Duplicate mark", "dup"],
        comment: ["Comment", "Comment", "cmt"]
    })
    function kindLabel(kind, rejected, byReporter) {
        var k = kinds[kind]
        return !k ? kind : rejected ? k[1] + " rejected" : !byReporter && k[3] ? k[3] : k[0]
    }
    // Inline monospace for codes, fingerprints and raw kinds (in a RichText label).
    function mono(s) { return "<span style='font-family:Menlo,Consolas,monospace;font-size:" + smallSize + "pt;color:" + t.muted + "'>" + esc(s) + "</span>" }
    // The card's grey line: age, whose move it is, next step. (Reopens and flags have their own chip and line.)
    function hint(i) {
        var waiting = i.status !== "AwaitingConfirmation" ? ""
            : needsMe(i) ? "Needs your confirmation. "
            : "Fix claimed: waiting for " + (i.claimant === i.reporter ? "2 residents" : "the reporter") + " to confirm. "
        return "Reported " + age(i.reported_ts) + ". " + waiting + nextLine(i)
    }
    // "Next: fit washer · due in 5 h"
    function nextLine(i) {
        var p = i.progress
        if (!p) return i.status === "Open" ? "Waiting for a steward to acknowledge" : ""
        var t = p.next_step ? "Next: " + p.next_step : p.note
        if (p.due_ts > 0 && isActionable(i.status) && dueHours(p) >= 0) t += " · due in " + dueHours(p) + " h"
        return t + " (" + who(p.by, p.by_name) + ")"
    }
    // Pick list for "Duplicate of…": every other issue that isn't closed yet.
    function duplicateTargets(id) {
        return issues.filter(function (o) {
            return o.id !== id && ["ConfirmedResolved", "ClosedWontfix", "Duplicate"].indexOf(o.status) < 0
        }).map(function (o) { return { id: o.id, label: place(o) + " · " + String(o.text).substr(0, 40) } })
    }
    function groupsOf(ls) {
        var g = []
        ls.forEach(function (l) { if (g.indexOf(l.group) < 0) g.push(l.group) })
        return g
    }
    // THE place filter, shared by the Report picker and the Members list: keeps the places that
    // match `query` (code, name or group), `group` ("" = any) and the retired switch, ordered so
    // every group is one run of rows (views print one header per run).
    function placesView(query, group, withRetired) {
        var q = query.trim().toLowerCase()
        var shown = locations.filter(function (l) {
            return (withRetired || l.state === "active") && (group === "" || l.group === group)
                && (q === "" || (l.code + " " + l.label + " " + l.group).toLowerCase().indexOf(q) >= 0)
        })
        var out = []
        groupNames.forEach(function (g) { shown.forEach(function (l) { if (l.group === g) out.push(l) }) })
        return out
    }
    function placeLabel(code) {
        var l = locations.filter(function (x) { return x.code === code })[0]
        return l ? l.code + "  " + l.label : code
    }
    function openIssuesText(n) { return plural(n, "open issue") }

    // ---- icons: the mockup's 20x20 stroke set (1.6 stroke, round caps), as SVG path data.
    // `icons` is stroked, `iconFills` adds a filled part, `iconDashes` a dashed one. ----
    function circ(cx, cy, r) { return "M" + (cx - r) + " " + cy + "a" + r + " " + r + " 0 1 0 " + 2 * r + " 0a" + r + " " + r + " 0 1 0 " + -2 * r + " 0" }
    function box(x, y, w, h, r) {
        const a = "a" + r + " " + r + " 0 0 1 "
        return "M" + (x + r) + " " + y + "h" + (w - 2 * r) + a + r + " " + r + "v" + (h - 2 * r) + a + -r + " " + r
             + "h" + (2 * r - w) + a + -r + " " + -r + "v" + (2 * r - h) + a + r + " " + -r + "z"
    }
    readonly property var icons: ({
        board: box(2, 3, 4.5, 14, 1) + box(7.75, 3, 4.5, 10, 1) + box(13.5, 3, 4.5, 7, 1),
        report: "M3 8.5v3h2.5l6 4v-11l-6 4z" + "M14.5 7.5a3.5 3.5 0 0 1 0 5",
        members: circ(7.5, 7, 3) + "M2 17c.6-3 2.8-4.5 5.5-4.5S12.4 14 13 17" + circ(14, 6.5, 2.3) + "M14.5 11c1.9.3 3 1.7 3.5 4",
        anchor: circ(10, 4.5, 2) + "M10 6.5V17M6.5 9.5h7M3.5 11.5a6.5 6.5 0 0 0 13 0",
        identity: circ(7, 10, 3.5) + "M10.5 10H18M15 10v3M17.5 10v2.2",
        lock: box(4.5, 9, 11, 8, 1.5) + "M7 9V6.5a3 3 0 0 1 6 0V9",
        signal: "M10 11.5v6" + circ(10, 10, 1.5) + "M6.5 6.5a5 5 0 0 0 0 7M13.5 6.5a5 5 0 0 1 0 7M4 4a8.5 8.5 0 0 0 0 12M16 4a8.5 8.5 0 0 1 0 12",
        user: circ(10, 6.5, 3.2) + "M3.5 17.5c.7-3.5 3.3-5.3 6.5-5.3s5.8 1.8 6.5 5.3",
        copy: box(7, 7, 10, 10, 1.5) + "M13 7V4.5A1.5 1.5 0 0 0 11.5 3h-7A1.5 1.5 0 0 0 3 4.5v7A1.5 1.5 0 0 0 4.5 13H7",
        dup: box(3, 3, 10, 10, 1.5) + box(7, 7, 10, 10, 1.5),
        chev: "M7.5 4.5l5.5 5.5-5.5 5.5", chevl: "M12.5 4.5L7 10l5.5 5.5", chevd: "M5 8l5 5 5-5",
        back: "M16 10H4.5M9 5l-5 5 5 5", x: "M5 5l10 10M15 5L5 15", plus: "M10 4v12M4 10h12", tick: "M5 10.5l3.2 3.2L15 6.5",
        warn: "M10 2.5l8 14H2z" + "M10 8v4M10 14.2v.1",
        info: circ(10, 10, 7.5) + "M10 9v5M10 6.2v.1",
        bell: "M5 14V9a5 5 0 0 1 10 0v5l1.5 1.5h-13z" + "M8.5 17.5a1.6 1.6 0 0 0 3 0",
        rep: circ(10, 10, 7.5), prog: circ(10, 10, 7.5), await: "M10 6v4.5l3 1.8",
        res: circ(10, 10, 7.5) + "M6.5 10.3l2.4 2.4 4.6-5",
        clo: circ(10, 10, 7.5) + "M5.2 14.8l9.6-9.6",
        reject: circ(10, 10, 7.5) + "M7 7l6 6M13 7l-6 6",
        radio: circ(10, 10, 7), radioOn: circ(10, 10, 7),
        cmt: "M3 4h14v9.5H9.5L5.5 17v-3.5H3z",
        reopen: "M4 10a6 6 0 1 0 2-4.5" + "M4 3v4h4",
        flag: "M5 18V3M5 3.5h10l-2.5 3.5L15 10.5H5",
        eye: "M1.5 10S4.5 4.5 10 4.5 18.5 10 18.5 10 15.5 15.5 10 15.5 1.5 10 1.5 10z" + circ(10, 10, 2.5),
        update: "M16 4v4h-4" + "M15.5 8A6 6 0 1 0 16 12",
        search: circ(8.5, 8.5, 5) + "M12.5 12.5L17 17",
        shield: "M10 2.5l6.5 2.5v5c0 4-3 6.5-6.5 7.5C6.5 16.5 3.5 14 3.5 10V5z",
        mega: "M3 8v4h2l7 4V4L5 8z" + "M15 7.5v5",
        hist: "M3.5 10a6.5 6.5 0 1 0 1.9-4.6" + "M3.5 3.5v3h3" + "M10 6.5V10l2.5 1.5",
        pin: "M10 18s-5.5-5.2-5.5-9.5a5.5 5.5 0 0 1 11 0C15.5 12.8 10 18 10 18z" + circ(10, 8.5, 2)
    })
    readonly property var iconFills: ({ rep: circ(10, 10, 2), prog: "M10 5a5 5 0 0 1 0 10z", radioOn: circ(10, 10, 3.5) })
    readonly property var iconDashes: ({ await: circ(10, 10, 7.5) })
    // A named icon, `size` px square, drawn in `color`.
    component Icon: Item {
        id: icon
        property string name
        property real size: 16
        property color color: root.t.fg
        implicitWidth: size; implicitHeight: size
        Accessible.ignored: true
        Shape {
            width: 20; height: 20
            scale: icon.size / 20; transformOrigin: Item.TopLeft
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: icon.color; strokeWidth: 1.6; fillColor: "transparent"
                capStyle: ShapePath.RoundCap; joinStyle: ShapePath.RoundJoin
                PathSvg { path: root.icons[icon.name] || "" }
            }
            ShapePath { strokeColor: "transparent"; fillColor: icon.color; PathSvg { path: root.iconFills[icon.name] || "" } }
            ShapePath {     // dash and gap are in stroke widths: 3 and 2.2 px of the 20 px box
                strokeColor: icon.color; strokeWidth: 1.6; fillColor: "transparent"
                capStyle: ShapePath.RoundCap; strokeStyle: ShapePath.DashLine; dashPattern: [1.9, 1.4]
                PathSvg { path: root.iconDashes[icon.name] || "" }
            }
        }
    }

    // ---- controls: one frame (border token, blue when focused) shared by every control, so no
    // field, button, combo or box is ever borderless whatever style the host uses ----
    function ink(on) { return on ? t.fg : t.disabled }       // text colour of a control
    component Frame: Rectangle {
        property bool ring: false          // keyboard/typing focus: thicker, blue
        property bool off: false           // a disabled control: flat, quiet border
        property color fill: off ? root.t.well : root.t.field
        property color edge: off ? root.t.bd : root.t.ctlBd
        implicitHeight: 36                 // the minimum touch target of every control
        color: fill
        radius: 6
        border.width: ring ? 2 : 1
        border.color: ring ? root.t.focus : edge
    }
    component FramedField: TextField {
        id: field
        selectByMouse: true
        leftPadding: 10; rightPadding: 10; topPadding: 6; bottomPadding: 6
        color: root.ink(enabled)
        placeholderTextColor: root.t.ph
        background: Frame { ring: field.activeFocus; off: !field.enabled }
    }
    // kind: "default", "primary" (the one main action) or "danger" (red text). Label wraps.
    component FramedButton: Button {
        id: fb
        property string kind: "default"
        readonly property bool solid: kind === "primary" && enabled
        leftPadding: 14; rightPadding: 14; topPadding: 6; bottomPadding: 6
        font.weight: Font.Medium
        contentItem: Label {
            text: fb.text; font: fb.font; wrapMode: Text.Wrap
            horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
            color: !fb.enabled ? root.t.disabled : fb.solid ? root.t.onPrimary : fb.kind === "danger" ? root.t.danger : root.t.fg
        }
        background: Frame {
            ring: fb.visualFocus
            fill: !fb.enabled ? root.t.btnOff : fb.solid ? (fb.hovered || fb.down ? root.t.primaryHover : root.t.primary)
                  : fb.down ? Qt.darker(root.t.btn, 1.12) : fb.hovered ? root.t.hover : root.t.btn
            edge: !fb.enabled ? root.t.bd : fb.solid ? root.t.primary : root.t.btnBd
        }
    }
    component FramedCombo: ComboBox {
        id: combo
        leftPadding: 10; rightPadding: 32; topPadding: 6; bottomPadding: 6
        background: Frame { ring: combo.visualFocus; off: !combo.enabled }
        contentItem: Label { text: combo.displayText; color: root.ink(combo.enabled); elide: Text.ElideRight; verticalAlignment: Text.AlignVCenter }
        indicator: Icon { x: combo.width - width - 10; y: (combo.height - height) / 2; name: "chevd"; size: 14; color: combo.enabled ? root.t.muted : root.t.disabled }
        // The style's own row paints its highlight from palette roles that are not meant for it
        // (invisible text on some palettes); this one reads the tokens.
        delegate: ItemDelegate {
            id: option
            required property var model
            required property int index
            width: ListView.view.width
            text: model[combo.textRole]
            highlighted: combo.highlightedIndex === index
            background: Rectangle { color: option.highlighted ? root.t.hover : root.t.field }
            contentItem: Label { text: option.text; verticalAlignment: Text.AlignVCenter; font.bold: combo.currentIndex === option.index }
        }
    }
    component FramedCheck: CheckBox {
        id: check
        implicitHeight: Math.max(36, implicitContentHeight + topPadding + bottomPadding)
        indicator: Frame {
            ring: check.visualFocus
            fill: !check.checked ? (check.enabled ? root.t.field : root.t.well) : check.enabled ? root.t.primary : root.t.disabled
            edge: check.checked ? fill : check.enabled ? root.t.ctlBd : root.t.bd
            implicitHeight: 18
            x: check.leftPadding; y: (check.height - height) / 2; width: 18; height: 18
            Icon { anchors.centerIn: parent; visible: check.checked; name: "tick"; size: 16; color: root.t.onPrimary }
        }
        contentItem: Label { text: check.text; color: root.ink(check.enabled); leftPadding: check.indicator.width + 8; verticalAlignment: Text.AlignVCenter }
    }
    component PageTab: TabButton {
        id: tab
        background: Frame {
            fill: tab.checked ? root.t.page : root.t.well
            Rectangle { visible: tab.checked; width: parent.width; height: 3; anchors.bottom: parent.bottom; color: root.t.accent }
        }
        contentItem: Label {
            text: tab.text; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
            font.bold: tab.checked
        }
        opacity: enabled ? 1 : 0.5
    }

    // ---- reusable pieces (props in, signals out; no reach into the enclosing scope) ----

    // Pill with an icon and text: a stage chip, "Reopened N×" (neutral) or "Needs your confirmation"
    // (filled blue). The text wraps, never elides, so long labels and Hindi stay whole. Colours
    // are a stage {fg, bg, bd}; the parent caps the width with maxWidth.
    component StatusChip: Rectangle {
        id: chip
        required property string label
        required property var stage     // stageStyle(): {fg, bg, bd, edge}
        property string icon: ""
        property bool strong: false
        property real maxWidth: 1e6
        readonly property real inset: 8 + 13 + 5
        width: Math.min(chipText.implicitWidth + inset + 8, maxWidth)
        height: Math.max(22, chipText.height + 2)
        radius: 12
        color: chip.stage.bg
        border.width: 1; border.color: chip.stage.bd
        Icon { x: 8; y: (chip.height - 13) / 2; name: chip.icon; size: 13; color: chip.stage.fg }
        Label {
            id: chipText
            x: chip.inset; y: (chip.height - height) / 2; width: chip.width - chip.inset - 8
            wrapMode: Text.Wrap; text: chip.label; color: chip.stage.fg
            font.pointSize: root.smallSize; font.weight: chip.strong ? Font.DemiBold : Font.Medium
        }
    }

    // One board card. All text is pre-computed by the caller (place/hint/flags/stage),
    // so this stays a pure presentational piece reusable across all four columns.
    component IssueCard: ItemDelegate {
        id: card
        required property var issue
        required property string placeText
        required property string hintText
        required property var flags          // red lines: root.flagLines()
        required property bool needsMe       // the viewer is the reporter of a claimed fix
        required property bool current       // the issue open in the timeline pane: gets the stage strip
        required property var stage
        signal opened()
        readonly property color ink: issue.location_retired ? root.t.muted : root.t.fg   // a retired place's card is quieter
        width: ListView.view.width
        topPadding: 10; bottomPadding: 12; rightPadding: 12
        leftPadding: current ? 16 : 12
        background: Frame {
            ring: card.visualFocus
            fill: card.down ? Qt.darker(root.t.card, 1.06) : card.issue.location_retired ? root.t.well : root.t.card
            edge: card.current || card.hovered ? root.t.ctlBd : root.t.bd
            Shape {   // 5 px strip, round on its outer corners only
                visible: card.current
                x: -1; y: -1; width: 5; height: card.height + 2
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    strokeColor: "transparent"; fillColor: card.stage.edge
                    PathSvg { path: "M5 0H6A6 6 0 0 0 0 6V" + (card.height - 4) + "A6 6 0 0 0 6 " + (card.height + 2) + "H5Z" }
                }
            }
        }
        Accessible.name: card.placeText + ": " + card.issue.text
        Accessible.description: card.issue.stage + ". " + card.hintText + " " + card.flags.join(" ")
        contentItem: Column {
            spacing: 6
            Label {
                width: parent.width; wrapMode: Text.Wrap; textFormat: Text.RichText; color: card.ink; font.weight: Font.DemiBold
                text: root.mono(root.placeCode(card.issue)) + " " + root.esc(root.placeName(card.issue))
            }
            Label { width: parent.width; wrapMode: Text.Wrap; color: card.ink; font.pointSize: root.baseSize * 1.07; text: card.issue.text }
            Flow {
                width: parent.width; spacing: 4
                StatusChip { label: card.issue.stage; stage: card.stage; icon: root.stageIcon(card.issue.status); maxWidth: parent.width }
                StatusChip {
                    visible: card.issue.reopen_count > 0; maxWidth: parent.width
                    label: "Reopened " + card.issue.reopen_count + "×"; stage: root.t.closed; icon: "reopen"
                }
                StatusChip {
                    visible: card.needsMe; maxWidth: parent.width; strong: true
                    label: "Needs your confirmation"; icon: "bell"
                    stage: ({ fg: root.t.onPrimary, bg: root.t.primary, bd: root.t.primary })
                }
            }
            Label { width: parent.width; wrapMode: Text.Wrap; font.pointSize: root.smallSize; color: root.t.muted; text: card.hintText }
            Repeater {
                model: card.flags
                delegate: Row {
                    id: flag
                    required property string modelData
                    width: card.availableWidth; spacing: 6
                    Icon { y: 2; name: "warn"; size: 15; color: root.t.danger }
                    Label {
                        width: flag.width - 21; wrapMode: Text.Wrap; text: flag.modelData
                        font.pointSize: root.smallSize; font.weight: Font.DemiBold; color: root.t.danger
                    }
                }
            }
        }
        onClicked: card.opened()
    }

    // A board column's title: stage icon, name, count pill, and a one-line subtitle.
    component ColumnHeader: Column {
        id: head
        required property string title
        required property string subtitle
        required property string icon
        required property color tint
        required property int count
        width: parent.width; spacing: 2; bottomPadding: 8
        Flow {
            width: parent.width; spacing: 8
            Row {
                spacing: 8
                Icon { anchors.verticalCenter: parent.verticalCenter; name: head.icon; size: 16; color: head.tint }
                Label { text: head.title; font.weight: Font.DemiBold }
            }
            Rectangle {
                width: Math.max(22, countText.implicitWidth + 14); height: 20; radius: 10; color: root.t.closed.bg
                Label { id: countText; anchors.centerIn: parent; text: head.count; font.pointSize: root.smallSize; font.weight: Font.DemiBold }
            }
        }
        Label { width: parent.width; wrapMode: Text.Wrap; text: head.subtitle; font.pointSize: root.smallSize; color: root.t.muted }
    }

    // One timeline entry (an issue_timeline() event): a node on the spine, the plain label then the
    // raw kind, who and when, the body, and the anchor line. A rejected event is red and stays in the record.
    component TimelineRow: RowLayout {
        id: row
        required property var event
        required property bool byReporter
        readonly property bool rejected: !!event.rejected
        readonly property var kind: root.kinds[event.kind] || []
        spacing: 10
        Rectangle {
            Layout.alignment: Qt.AlignTop
            implicitWidth: 28; implicitHeight: 28; radius: 14
            color: row.rejected ? root.t.dangerBg : root.t.well
            border.color: row.rejected ? root.t.dangerBd : root.t.bd
            Icon { anchors.centerIn: parent; name: row.rejected ? "reject" : row.kind[2] || "cmt"; size: 16; color: row.rejected ? root.t.danger : root.t.muted }
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            Label {
                Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.RichText
                color: row.rejected ? root.t.danger : root.t.fg
                text: "<b>" + root.esc(root.kindLabel(row.event.kind, row.rejected, row.byReporter)) + "</b>&nbsp; " + root.mono(row.event.kind)
            }
            RowLayout {
                Layout.fillWidth: true
                spacing: 5
                Icon { Layout.alignment: Qt.AlignTop; Layout.topMargin: 2; name: "user"; size: 13; color: root.t.muted }
                Label {
                    Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.RichText; font.pointSize: root.smallSize; color: root.t.muted
                    text: root.esc(root.who(row.event.author, row.event.author_name)) + " " + root.mono(String(row.event.author).substr(0, 6))
                          + " · " + root.when(row.event.ts)
                }
            }
            Label { visible: text !== ""; Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.PlainText; text: row.event.body || "" }
            Label {
                visible: row.rejected; Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.PlainText
                text: "[rejected: " + row.event.rejected + "]"; color: root.t.danger; font.weight: Font.DemiBold
            }
            Label { visible: row.rejected; text: "Kept in the record. It changed nothing."; font.pointSize: root.smallSize; color: root.t.muted }
            Label {
                visible: !!row.event.anchored_tx; Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.RichText
                font.pointSize: root.smallSize; color: root.t.muted
                text: "anchor ref " + root.mono(String(row.event.anchored_tx).substr(0, 16) + "…") + " recorded by "
                      + root.esc(root.who(row.event.anchored_by, row.event.anchored_by_name))
            }
        }
    }

    // "Why it isn't closed yet" (proposal F): the steps root.whyCard() returns, tinted by the issue's stage.
    component WhyCard: Rectangle {
        id: why
        required property var card      // whyCard(): {intro, steps: [{title, hint, meter?, done?}], never}
        required property var stage     // stageStyle(): header tint, icon and border
        implicitHeight: whyCol.implicitHeight + 2
        radius: 8; color: root.t.card; border.color: why.stage.bd
        Accessible.role: Accessible.Grouping; Accessible.name: "Why it isn't closed yet"
        ColumnLayout {
            id: whyCol
            x: 1; y: 1; width: parent.width - 2
            spacing: 0
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: whyHead.implicitHeight + 16
                color: why.stage.bg; topLeftRadius: 7; topRightRadius: 7
                RowLayout {
                    id: whyHead
                    anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; margins: 12 }
                    spacing: 8
                    Icon { name: "info"; size: 16; color: why.stage.fg }
                    Label { Layout.fillWidth: true; wrapMode: Text.Wrap; text: "Why it isn't closed yet"; font.weight: Font.DemiBold }
                }
            }
            ColumnLayout {
                Layout.fillWidth: true; Layout.margins: 12; Layout.topMargin: 10; Layout.bottomMargin: 10
                spacing: 0
                Label { visible: why.card.intro !== ""; Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.RichText; text: why.card.intro; bottomPadding: 6 }
                Repeater {
                    model: why.card.steps
                    delegate: ColumnLayout {
                        id: step
                        required property var modelData
                        required property int index
                        readonly property bool dashed: index > 0 || why.card.intro !== ""
                        Layout.fillWidth: true
                        spacing: 0
                        Shape {   // dashed rule between the steps: a long line, clipped to the card
                            visible: step.dashed
                            Layout.fillWidth: true; implicitHeight: 1; clip: true
                            ShapePath {
                                strokeColor: root.t.bd; strokeWidth: 1; strokeStyle: ShapePath.DashLine; dashPattern: [3, 3]
                                startX: 0; startY: 0.5
                                PathLine { x: 2000; y: 0.5 }
                            }
                        }
                        RowLayout {
                            Layout.fillWidth: true; Layout.topMargin: step.dashed ? 8 : 0; Layout.bottomMargin: 8
                            spacing: 10
                            Rectangle {
                                Layout.alignment: Qt.AlignTop
                                implicitWidth: 24; implicitHeight: 24; radius: 12; color: "transparent"
                                border.width: 2; border.color: step.modelData.done ? root.t.resolved.fg : root.t.ctlBd
                                Label {
                                    anchors.centerIn: parent; text: step.index + 1; font.pointSize: root.smallSize; font.bold: true
                                    color: step.modelData.done ? root.t.resolved.fg : root.t.fg
                                }
                            }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Label { Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.RichText; text: step.modelData.title }
                                Rectangle {   // the residents' votes so far, of 2
                                    visible: step.modelData.meter !== undefined
                                    Layout.fillWidth: true; Layout.maximumWidth: 160; Layout.topMargin: 4; implicitHeight: 8
                                    radius: 4; color: root.t.bd
                                    Accessible.role: Accessible.Indicator; Accessible.name: step.modelData.hint
                                    Rectangle { width: parent.width * Math.min(step.modelData.meter || 0, 2) / 2; height: 8; radius: 4; color: root.t.resolved.fg }
                                }
                                Label {
                                    visible: text !== ""; Layout.fillWidth: true; wrapMode: Text.Wrap
                                    text: step.modelData.hint; font.pointSize: root.smallSize; color: root.t.muted
                                }
                            }
                        }
                    }
                }
                Rectangle {   // the fixer never confirms their own fix
                    visible: why.card.never !== ""
                    Layout.fillWidth: true
                    implicitHeight: neverRow.implicitHeight + 12
                    radius: 6; color: root.t.dangerBg; border.color: root.t.dangerBd
                    RowLayout {
                        id: neverRow
                        anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; leftMargin: 10; rightMargin: 10 }
                        spacing: 8
                        Icon { Layout.alignment: Qt.AlignTop; Layout.topMargin: 1; name: "warn"; size: 15; color: root.t.danger }
                        Label { Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.RichText; text: why.card.never; font.pointSize: root.smallSize }
                    }
                }
            }
        }
    }

    // A muted heading over a wrapping row of actions: "For stewards", "For everyone", ...
    component ActionGroup: ColumnLayout {
        id: group
        property string title
        default property alias content: acts.data
        Layout.fillWidth: true
        spacing: 6
        Label { Layout.topMargin: 6; text: group.title; font.pointSize: root.smallSize; font.weight: Font.DemiBold; color: root.t.muted }
        Flow { id: acts; Layout.fillWidth: true; spacing: 8 }
    }

    // ---- layout pieces ----

    // One tab of the StackLayout: scrolls when its content outgrows the window (30 pending
    // members, a long checkpoint). Content is a column of the page's width, capped so forms
    // don't sprawl across a wide window.
    component TabPage: ScrollView {
        id: page
        default property alias content: column.data
        contentWidth: availableWidth
        clip: true
        ColumnLayout { id: column; width: Math.min(page.availableWidth, 760); spacing: 8 }
    }

    // A bold section title.
    component Heading: Label {
        Layout.topMargin: 8
        font.bold: true
        font.pointSize: root.baseSize * 1.15
    }

    // A section the reader opens on demand: a full-width header (chevron + title) that is one
    // button (Space or Enter toggles it), then whatever is put inside. `open` lives only as long
    // as the page, so it is remembered for the session and nothing more.
    component CollapsibleSection: ColumnLayout {
        id: section
        property string title
        property bool open: false
        default property alias content: body.data
        signal toggled()
        Layout.fillWidth: true
        spacing: 8
        ItemDelegate {
            id: header
            Layout.fillWidth: true
            Layout.topMargin: 8
            text: section.title
            Accessible.role: Accessible.Button
            Accessible.name: section.title + (section.open ? ", expanded" : ", collapsed")
            onClicked: { section.open = !section.open; section.toggled() }
            Keys.onReturnPressed: clicked()
            Keys.onEnterPressed: clicked()
            background: Frame { ring: header.visualFocus; fill: header.hovered ? root.t.hover : root.t.well }
            contentItem: RowLayout {
                spacing: 8
                Label { text: section.open ? "\u25be" : "\u25b8"; font.pointSize: root.baseSize * 1.15 }
                Label { Layout.fillWidth: true; text: header.text; elide: Text.ElideRight; font.bold: true; font.pointSize: root.baseSize * 1.15 }
            }
        }
        ColumnLayout { id: body; visible: section.open; Layout.fillWidth: true; spacing: 8 }
    }

    // A whole-number field with its one-line explanation underneath.
    component RuleField: FormRow {
        id: rule
        property alias text: field.text
        property string hint
        FramedField {
            id: field
            Layout.preferredWidth: 120
            maximumLength: 6
            inputMethodHints: Qt.ImhDigitsOnly
            validator: RegularExpressionValidator { regularExpression: /[0-9]*/ }
            Accessible.name: rule.label
        }
        Note { text: rule.hint; color: root.t.muted; font.pointSize: root.smallSize }
    }

    // A visible label above whatever is put inside (every field has one, not only a placeholder).
    component FormRow: ColumnLayout {
        id: form
        property string label
        property string sub          // a muted aside after a bold label: "Note (what was seen, done or why)"
        property bool strong: sub !== ""   // the pane's bold 14 px labels (step 6 brings the rest)
        Layout.fillWidth: true
        spacing: 4
        Label {
            visible: form.label !== ""; Layout.fillWidth: true; wrapMode: Text.Wrap
            textFormat: Text.RichText; font.weight: form.strong ? Font.DemiBold : Font.Normal; font.pointSize: form.strong ? root.baseSize : root.smallSize
            text: root.esc(form.label) + (form.sub ? " <span style='font-weight:400;font-size:" + root.smallSize + "pt;color:" + root.t.muted + "'>" + root.esc(form.sub) + "</span>" : "")
        }
    }

    // A full-width notice: wrapped text, and optionally one button on the right.
    component Banner: RowLayout {
        id: banner
        property string text
        property color tint: root.t.fg
        property bool bold: false
        property string buttonText
        property string buttonName: buttonText     // what a screen reader says for the button
        signal activated()
        Layout.fillWidth: true
        Label { id: label; Layout.fillWidth: true; wrapMode: Text.Wrap; text: banner.text; color: banner.tint; font.bold: banner.bold }
        ActionButton {
            visible: banner.buttonText !== ""
            text: banner.buttonText
            Accessible.name: banner.buttonName
            onClicked: banner.activated()
        }
    }

    // A wrapped paragraph. Plain text on purpose: "<YOUR_PUBLIC_ACCOUNT>" must not be read as a tag.
    component Note: Label {
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        textFormat: Text.PlainText
    }

    // The one multi-line text box: visible frame, scrolls, optional placeholder. Read-only boxes
    // are selectable and get a Copy button.
    component FramedTextArea: ColumnLayout {
        id: fta
        property alias text: area.text
        property alias placeholder: area.placeholderText
        property bool readOnly: false
        property string name              // what a screen reader says for the box
        property int boxHeight: 70
        Layout.fillWidth: true
        spacing: 4
        ScrollView {
            Layout.fillWidth: true
            Layout.preferredHeight: fta.boxHeight
            clip: true
            background: Frame { ring: area.activeFocus }
            TextArea {
                id: area
                readOnly: fta.readOnly
                selectByMouse: true
                wrapMode: TextEdit.WrapAnywhere
                placeholderTextColor: root.t.ph
                padding: 8
                background: null
                Accessible.name: fta.name
            }
        }
        FramedButton {
            visible: fta.readOnly && area.text !== ""
            text: "Copy"
            Accessible.name: "Copy " + fta.name
            onClicked: root.copyText(area.text)
        }
    }

    // A button that also waits while a call is running. `allowed` is the caller's own
    // precondition; submit() is what Return in a neighbouring field calls (it honours both).
    component ActionButton: FramedButton {
        property bool allowed: true
        property bool shown: true   // false hides it but keeps its slot, so a row's columns line up
        enabled: shown && allowed && !root.busy
        opacity: shown ? 1 : 0
        Accessible.ignored: !shown
        function submit() { if (enabled) clicked() }
    }

    // A text field with its submit button glued on: the "paste id / type name, then act" row.
    // `label` sits above it; `caption` is a left-hand description for list rows (field then keeps a fixed width).
    component LabelledField: FormRow {
        id: lf
        property alias text: field.text
        property alias placeholder: field.placeholderText
        property alias echoMode: field.echoMode
        property alias fieldEnabled: field.enabled
        property string name: label
        property string caption
        property color captionColor: root.t.fg
        property int indent: 0              // side margin, for rows inside a framed list
        property string buttonText: "Go"
        property bool buttonEnabled: true
        signal submitted()
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: lf.indent
            Layout.rightMargin: lf.indent
            spacing: 8
            Label { visible: lf.caption !== ""; Layout.fillWidth: true; elide: Text.ElideRight; text: lf.caption; color: lf.captionColor; font.family: "monospace"; textFormat: Text.PlainText }
            FramedField {
                id: field
                Layout.fillWidth: lf.caption === ""
                Layout.preferredWidth: lf.caption === "" ? -1 : 200
                maximumLength: 500
                Accessible.name: lf.name
                onAccepted: go.submit()
            }
            ActionButton { id: go; Layout.preferredWidth: lf.caption === "" ? -1 : 90; text: lf.buttonText; allowed: lf.buttonEnabled; onClicked: lf.submitted() }
        }
    }

    // Group title inside a place list.
    component SectionHeader: Rectangle {
        required property string section
        width: ListView.view.width
        height: title.implicitHeight + 8
        color: root.t.well
        Label { id: title; x: 8; anchors.verticalCenter: parent.verticalCenter; text: parent.section; font.bold: true; color: root.t.muted }
    }

    // One selectable row of the Report picker. A retired place is greyed and never becomes the
    // pick; with `explains` it stays clickable so the caller can say why (see PlaceBrowser.retiredPlace).
    component PlaceChoice: ItemDelegate {
        id: choice
        property bool picked: false
        property bool retired: false
        property bool explains: false
        topPadding: 5; bottomPadding: 5
        enabled: !retired || explains
        opacity: retired ? 0.55 : 1
        Accessible.role: Accessible.RadioButton
        Accessible.checked: choice.picked
        background: Rectangle { color: choice.picked ? root.t.accentBg : choice.hovered ? root.t.hover : root.t.page }
        contentItem: Label { text: choice.text; elide: Text.ElideRight; font.bold: choice.picked; font.italic: choice.retired }
    }

    // Filters (search, group, retired) + a count + a framed list of the matching places, grouped
    // under headers. The caller draws the rows (`rowDelegate`, may be given `listHeader` above them).
    component PlaceBrowser: ColumnLayout {
        id: pb
        property int maxHeight: 260
        property Component rowDelegate
        property Component listHeader
        // Report picker only: the retired place the user just tried to pick (null = none). While
        // set, a notice under the list explains it and offers to report it as "Other".
        property var retiredPlace: null
        signal reportOther()
        readonly property var places: root.placesView(search.text, groupFilter.currentIndex > 0 ? groupFilter.currentText : "", showRetired.checked)
        readonly property int total: root.placesView("", "", showRetired.checked).length
        Layout.fillWidth: true
        spacing: 6
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            FormRow {
                label: "Search"
                FramedField { id: search; Layout.fillWidth: true; placeholderText: "code, name or group"; Accessible.name: "Search places" }
            }
            FormRow {
                label: "Group"
                Layout.fillWidth: false
                FramedCombo {
                    id: groupFilter
                    Layout.preferredWidth: 200
                    Accessible.name: "Filter by group"
                    model: ["All groups"].concat(root.groupNames)
                }
            }
            FramedCheck { id: showRetired; text: "Show retired places"; Layout.alignment: Qt.AlignBottom; Layout.bottomMargin: 4 }
        }
        Label {
            font.pointSize: root.smallSize
            color: root.t.muted
            text: pb.places.length === 0 ? "No places match. Clear the search or pick another group."
                                         : pb.places.length + " of " + pb.total + " places"
        }
        Frame {
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(list.contentHeight, pb.maxHeight) + 2
            ListView {
                id: list
                anchors.fill: parent
                anchors.margins: 1
                clip: true
                spacing: 2
                cacheBuffer: 100000      // keep every row alive: text typed into a row survives scrolling
                model: pb.places
                delegate: pb.rowDelegate
                section.property: "group"
                section.delegate: SectionHeader {}
                header: Loader { width: list.width; sourceComponent: pb.listHeader }
                ScrollBar.vertical: ScrollBar {}
            }
        }
        Banner {
            visible: pb.retiredPlace !== null && showRetired.checked
            readonly property bool removing: !!pb.retiredPlace && pb.retiredPlace.state === "pending_removal"
            text: !pb.retiredPlace ? "" : pb.retiredPlace.code + " " + pb.retiredPlace.label
                + (removing ? " is being removed (" + root.placeState(pb.retiredPlace) + ")" : " is retired")
                + (pb.retiredPlace.retired_reason || pb.retiredPlace.removal_reason ? " (reason: " + (pb.retiredPlace.retired_reason || pb.retiredPlace.removal_reason) + ")" : "")
                + ". If the problem is real, report it as Other and describe the place, or ask the admin to "
                + (removing ? "undo the removal." : "restore it.")
            buttonText: "Report as Other at this spot"
            onActivated: pb.reportOther()
        }
    }

    Timer { interval: 2000; running: true; repeat: true; triggeredOnStart: true; onTriggered: root.refresh() }

    Rectangle { anchors.fill: parent; color: root.t.page }     // the page never shows the host's window colour

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            spacing: 12
            Label { text: "Pukaar"; font.pointSize: root.titleSize; font.bold: true }
            Label { Layout.fillWidth: true; elide: Text.ElideRight; text: root.inSite ? (root.info.name || "") : "not in a site yet"; color: root.t.muted }
            Label {   // Delivery bring-up status from pukaar_core; errors stay visible until it recovers
                Layout.maximumWidth: root.width / 2
                elide: Text.ElideRight
                text: "delivery: " + root.deliveryLabel(root.me.delivery)
                color: root.failed(root.me.delivery) ? root.t.danger : root.t.muted
            }
        }
        RowLayout {
            visible: root.inSite
            Layout.fillWidth: true
            spacing: 8
            Label {
                Layout.fillWidth: true
                elide: Text.ElideRight
                text: "you: " + (root.me.name || "pseudonym") + " · " + (root.me.fingerprint || "") + " · "
                      + (root.me.role || "pending: read your fingerprint at the kiosk")
            }
            // the 64-hex id is long: show the short form, Copy puts the full one on the clipboard
            Label { text: "site " + root.shortId(root.me.site || ""); font.pointSize: root.smallSize; color: root.t.muted }
            FramedButton {
                text: "Copy"
                Accessible.name: "Copy the full site id"
                onClicked: root.copyText(root.me.site)
            }
        }
        Banner {   // the last refused action; stays until dismissed or the next action replaces it
            visible: root.message !== ""
            text: root.message; tint: root.t.danger
            buttonText: "×"; buttonName: "Dismiss this error"
            onActivated: root.message = ""
        }
        Banner {   // forks: someone signed two versions of their log (the threat model says: flag it)
            visible: root.inSite && root.info.forks > 0
            tint: root.t.danger; bold: true
            text: "Integrity warning: " + root.info.forks + " conflicting history pair(s) found. Someone signed two versions of their log, so devices may show different boards."
        }
        Banner {   // joined, but the genesis hasn't arrived: wrong id, or nobody online yet
            visible: root.inSite && !root.info.name
            text: "Waiting for site data… If this lasts, check the site id, or wait for a member to come online."
            buttonText: "Leave this site"
            onActivated: root.leave()
        }
        Banner {
            visible: root.inSite && root.syncing
            tint: root.t.danger
            text: "Restoring your earlier reports from the network. Actions are paused until they arrive, so your new ones can't conflict with them."
            buttonText: "Skip waiting (history lost)"
            onActivated: root.run("skip_history_sync", [])
        }

        // ---------- first run: join, create or restore ----------
        TabPage {
            visible: !root.inSite
            Layout.fillWidth: true
            Layout.fillHeight: true
            Heading { text: "Join a site"; Layout.topMargin: 0 }
            LabelledField {
                id: joinField
                label: "Site id (shown on the kiosk)"
                placeholder: "64 hex characters, e.g. 45f1dbbf…a050"
                buttonText: "Join"
                onSubmitted: root.run("site_join", [joinField.text])
            }
            CollapsibleSection {
                title: "Create a site (site admin only)"
                Note { text: "Nothing is published until you press Create site. Places can be added later too, in Members." }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    FormRow {
                        label: "Site name"
                        FramedField { id: siteName; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Site name"; placeholderText: "e.g. Dhun relief camp" }
                    }
                    FormRow {   // staff are always named: this goes into the genesis as the admin's name
                        label: "Your name, as residents will see it"
                        FramedField { id: adminName; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Your name"; placeholderText: "e.g. John" }
                    }
                }
                FormRow {
                    label: "Categories (what can go wrong)"
                    Flow {
                        Layout.fillWidth: true
                        spacing: 6
                        Repeater {
                            model: setup.categories
                            delegate: FramedButton {
                                required property string modelData
                                required property int index
                                text: modelData + "  \u00d7"
                                Accessible.name: "Remove category " + modelData
                                onClicked: setup.categories = setup.categories.filter(function (c, i) { return i !== index })
                            }
                        }
                    }
                }
                LabelledField {
                    id: newCategory
                    label: "Add a category"
                    placeholder: "e.g. electricity"
                    buttonText: "Add"
                    buttonEnabled: setup.canAddCategory
                    onSubmitted: { setup.categories = setup.categories.concat([newCategory.text.trim()]); newCategory.text = "" }
                }
                FormRow {
                    label: "Places (" + setup.places.count + ")"
                    Note {
                        color: root.t.muted
                        text: "Each place has a group (for the picker), a short code people can say aloud, and a name. Codes can't change later."
                    }
                    RowLayout {   // column titles: the placeholders vanish once a row is filled
                        visible: setup.places.count > 0
                        spacing: 6
                        Label { Layout.preferredWidth: 170; text: "Group"; font.pointSize: root.smallSize; color: root.t.muted }
                        Label { Layout.preferredWidth: 90; text: "Code"; font.pointSize: root.smallSize; color: root.t.muted }
                        Label { text: "Name"; font.pointSize: root.smallSize; color: root.t.muted }
                    }
                    Repeater {
                        model: setup.places
                        delegate: RowLayout {
                            id: draftRow
                            required property int index
                            required property string group
                            required property string code
                            required property string label
                            Layout.fillWidth: true
                            spacing: 6
                            // Writes straight into the ListModel, so other rows keep their typed text.
                            FramedField { Layout.preferredWidth: 170; maximumLength: 500; text: draftRow.group; placeholderText: "e.g. Water points"; Accessible.name: "Place " + (draftRow.index + 1) + " group"; onTextEdited: setup.edit(draftRow.index, "group", text) }
                            FramedField { Layout.preferredWidth: 90; maximumLength: 500; text: draftRow.code; placeholderText: "e.g. W-01"; Accessible.name: "Place " + (draftRow.index + 1) + " code"; onTextEdited: setup.edit(draftRow.index, "code", text) }
                            FramedField { Layout.fillWidth: true; maximumLength: 500; text: draftRow.label; placeholderText: "e.g. Tap, dining hall"; Accessible.name: "Place " + (draftRow.index + 1) + " name"; onTextEdited: setup.edit(draftRow.index, "label", text) }
                            FramedButton { text: "\u00d7"; Accessible.name: "Remove place " + (draftRow.index + 1); onClicked: setup.places.remove(draftRow.index) }
                        }
                    }
                    Flow {
                        Layout.fillWidth: true
                        spacing: 6
                        FramedButton { text: "Add a place"; onClicked: setup.places.append({ group: "", code: "", label: "" }) }
                        FramedButton { text: "Start from the Dhun sample"; onClicked: setup.fillSample() }
                        FramedButton { visible: setup.places.count > 0; text: "Clear places"; onClicked: setup.places.clear() }
                    }
                }
                CollapsibleSection {
                    title: "Service rules (optional)"
                    RuleField { id: ruleAck; label: "Acknowledge within (hours)"; text: "12"; hint: "After this, an unacknowledged report is flagged red." }
                    RuleField { id: ruleFix; label: "Fix within (hours, after acknowledging)"; text: "48"; hint: "After this, the issue shows as overdue." }
                    RuleField { id: ruleMax; label: "Open reports per person (max)"; text: "10"; hint: "Stops one person flooding the board; more can be filed once some are resolved." }
                }
                CollapsibleSection {
                    id: advanced
                    title: "Advanced: edit the settings as JSON"
                    onToggled: if (open) advancedJson.text = setup.genesisJson()
                    Note { color: root.t.muted; text: "Starts from the form above and replaces it while this section is open. Close it to go back to the form." }
                    FramedTextArea { id: advancedJson; name: "Site settings, JSON"; boxHeight: 160 }
                }
                ActionButton {
                    text: "Create site"
                    allowed: advanced.open ? adminName.text.trim() !== "" : setup.problem === ""
                    onClicked: root.run("site_create", [advanced.open ? setup.withAdmin(advancedJson.text) : setup.genesisJson()], null,
                                        "Start with fewer places, then add the rest after creating, via Members \u2192 Add a location.")
                }
                Note { visible: !advanced.open && setup.problem !== ""; color: root.t.muted; text: setup.problem }
                // The draft and its checks. The core repeats every check (it is the authority); these
                // only say what's missing before anyone presses the button.
                QtObject {
                    id: setup
                    property var categories: ["water", "waste", "power", "access", "rooms", "kitchen", "safety"]
                    property ListModel places: ListModel {}
                    readonly property var rules: [ruleAck, ruleFix, ruleMax]
                property int edits: 0          // bumped on every change: ListModel.get() alone is not watched
                    function edit(i, role, text) { places.setProperty(i, role, text); edits++ }
                    readonly property bool canAddCategory: newCategory.text.trim() !== "" && categories.indexOf(newCategory.text.trim()) < 0
                    function fillSample() {
                        var sample = [["Water points", "W-01", "Tap, dining hall"], ["Water points", "W-02", "Tap, tent row A"],
                                      ["Bins", "B-01", "Bin, main path"], ["Paths", "P-01", "Gate to the farm path"],
                                      ["Rooms and tents", "R-01", "Tent 1"]]
                        places.clear()
                        sample.forEach(function (p) { places.append({ group: p[0], code: p[1], label: p[2] }) })
                        edits++
                    }
                    function draft() {
                        var out = []
                        for (var i = 0; i < places.count; i++) {
                            var p = places.get(i)
                            out.push({ code: p.code.trim(), label: p.label.trim(), group: p.group.trim() })
                        }
                        return out
                    }
                    // What still blocks Create site, in words ("" = ready). Recomputed on every edit.
                    readonly property string problem: {
                        if (siteName.text.trim() === "") return "Give the site a name."
                        if (adminName.text.trim() === "") return "Give your name: staff are always named."
                        if (categories.length === 0) return "Add at least one category."
                    for (var r = 0; r < rules.length; r++)
                        if (!/^[1-9][0-9]*$/.test(rules[r].text)) return "Service rules: " + rules[r].label.toLowerCase() + " needs a whole number, 1 or more."
                        var seen = {}
                        var ps = edits >= 0 && places.count >= 0 ? draft() : []      // reads both, so any change re-runs this
                        for (var i = 0; i < ps.length; i++) {
                            var p = ps[i], n = "Place " + (i + 1)
                            if (p.code === "") return n + " needs a code."
                            if (p.code.toLowerCase() === "other") return n + ": the code \u201cother\u201d is reserved for places not on the list."
                            if (seen[p.code]) return n + ": code " + p.code + " is already used by place " + seen[p.code] + "."
                            if (p.label === "" || p.group === "") return n + " (" + p.code + ") needs a group and a name."
                            seen[p.code] = i + 1
                        }
                        return ""
                    }
                    function genesisJson() {
                        return JSON.stringify({ Genesis: { name: siteName.text.trim(), admin_name: adminName.text.trim(), categories: categories,
                                                           locations: draft(), sla_ack_h: parseInt(ruleAck.text, 10),
                                                       sla_fix_h: parseInt(ruleFix.text, 10), max_open_per_author: parseInt(ruleMax.text, 10) } }, null, 1)
                    }
                    // The admin's name always comes from the field, never the JSON.
                    function withAdmin(json) {
                        try { var g = JSON.parse(json); g.Genesis.admin_name = adminName.text.trim(); return JSON.stringify(g) }
                        catch (e) { return json }      // the core reports the parse error
                    }
                }
            }
            CollapsibleSection {
                title: "Restore your identity"
                Note { text: "Paste the backup made in the Identity tab, on your other device or before you reinstalled." }
                FormRow {
                    label: "Identity backup"
                    FramedTextArea {
                        id: importBlob
                        name: "Identity backup"
                        placeholder: "e.g. pukaar-id-1:…"
                    }
                }
                LabelledField {
                    id: importPassword
                    label: "Backup password"
                    placeholder: "the password you set when exporting"
                    echoMode: TextInput.Password
                    buttonText: "Import identity"
                    buttonEnabled: importBlob.text.trim() !== "" && importPassword.text !== ""
                    onSubmitted: {
                        root.run("import_identity", [importBlob.text, importPassword.text],
                                 function () { importBlob.text = ""; importPassword.text = "" })
                    }
                }
            }
        }

        // ---------- in a site ----------
        TabBar {
            id: tabs
            visible: root.inSite
            Layout.fillWidth: true
            background: null
            PageTab { text: "Board" }
            PageTab { text: "Report" }
            PageTab { text: "Members"; enabled: root.isAdmin }
            PageTab { text: "Anchor" }
            PageTab { text: "Identity" }
        }

        StackLayout {
            visible: root.inSite
            currentIndex: tabs.currentIndex
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Board (the brief's three stages) + timeline
            ColumnLayout {
                FramedCheck { text: "Only my reports"; checked: root.onlyMine; onToggled: root.onlyMine = checked }
                Rectangle {   // an empty board says why, and what to do
                    visible: root.boardCount === 0
                    Layout.fillWidth: true; Layout.bottomMargin: 8
                    implicitHeight: emptyRow.implicitHeight + 20
                    radius: 6; color: root.t.accentBg; border.color: root.t.accentBd
                    RowLayout {
                        id: emptyRow
                        anchors { fill: parent; margins: 10 }
                        spacing: 10
                        Icon { name: "info"; size: 16; color: root.t.accent; Layout.alignment: Qt.AlignTop; Layout.topMargin: 2 }
                        Label {
                            Layout.fillWidth: true; wrapMode: Text.Wrap
                            text: root.onlyMine ? "You haven't reported anything yet. Use the Report tab to raise a problem."
                                                : "No reports yet. Use the Report tab to raise the first one."
                        }
                        FramedButton { text: "Report a problem"; onClicked: tabs.currentIndex = 1 }
                    }
                }
                RowLayout {
                    spacing: 8
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    RowLayout {   // the four columns; hidden behind the pane on a narrow window
                        visible: !(root.narrow && root.selected)
                        spacing: 12
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Repeater {
                            model: [
                                { title: "Reported", status: "Open", statuses: ["Open", "Acknowledged"], subtitle: "Waiting for a steward to see it", empty: "No reports yet." },
                                { title: "In progress", status: "InProgress", statuses: ["InProgress", "AwaitingConfirmation"], subtitle: "A steward is on it", empty: "Nothing in progress." },
                                { title: "Resolved", status: "ConfirmedResolved", statuses: ["ConfirmedResolved"], subtitle: "Confirmed fixed", empty: "Nothing resolved yet." },
                                { title: "Closed without fix", status: "ClosedWontfix", statuses: ["ClosedWontfix", "Duplicate"], subtitle: "Won't fix or duplicate", empty: "Nothing closed." }
                            ]
                            delegate: Rectangle {
                                id: col
                                required property var modelData
                                readonly property var cards: root.inColumn(modelData.statuses)
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                Layout.preferredWidth: 1   // equal shares of the width...
                                Layout.minimumWidth: 160   // ...but never thinner than this
                                color: root.t.well; radius: 8; border.color: root.t.bd
                                ColumnLayout {
                                    anchors { fill: parent; margins: 8 }
                                    spacing: 0
                                    ColumnHeader {
                                        Layout.fillWidth: true; Layout.topMargin: 4; Layout.leftMargin: 4
                                        title: col.modelData.title; subtitle: col.modelData.subtitle; count: col.cards.length
                                        icon: root.stageIcon(col.modelData.status); tint: root.stageStyle(col.modelData.status).fg
                                    }
                                    Label {   // an empty column says so, in a dashed box
                                        id: empty
                                        visible: col.cards.length === 0
                                        Layout.fillWidth: true; padding: 12; horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap
                                        text: root.onlyMine ? "None of your reports here yet." : col.modelData.empty
                                        font.pointSize: root.smallSize; color: root.t.muted
                                        background: Shape {
                                            preferredRendererType: Shape.CurveRenderer
                                            ShapePath {
                                                strokeColor: root.t.ctlBd; strokeWidth: 1; fillColor: "transparent"
                                                strokeStyle: ShapePath.DashLine; dashPattern: [4, 3]
                                                PathSvg { path: root.box(0.5, 0.5, empty.width - 1, empty.height - 1, 6) }
                                            }
                                        }
                                    }
                                    ListView {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        clip: true
                                        spacing: 8
                                        model: col.cards
                                        delegate: IssueCard {
                                            required property var modelData
                                            issue: modelData
                                            placeText: root.place(modelData)
                                            hintText: root.hint(modelData)
                                            flags: root.flagLines(modelData)
                                            needsMe: root.needsMe(modelData)
                                            current: !!root.selected && root.selected.issue.id === modelData.id
                                            stage: root.stageStyle(modelData.status)
                                            onOpened: root.openIssue(modelData.id)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // The issue pane: header, the report, why it isn't closed yet, the timeline, then the
                    // actions grouped by who they are for and what isn't available (and why). It scrolls whole.
                    ScrollView {
                        id: detail
                        visible: root.selected !== null
                        Layout.fillWidth: root.narrow   // wide: a fixed pane beside the board; narrow: it takes over
                        Layout.preferredWidth: root.narrow ? -1 : 400
                        Layout.minimumWidth: 260
                        Layout.fillHeight: true
                        contentWidth: availableWidth
                        clip: true
                        leftPadding: root.narrow ? 0 : 16
                        background: Rectangle {
                            color: root.t.page
                            Rectangle { visible: !root.narrow; width: 1; height: parent.height; color: root.t.bd }
                        }
                        property var issue: root.selected ? root.selected.issue : ({})
                        property string st: issue.status || ""
                        property bool mine: issue.reporter === root.me.key
                        readonly property var stage: root.stageStyle(st)
                        // The rules (root.stewardWhy/confirmWhy/reopenWhy): "" = this viewer may.
                        readonly property bool steward: root.stewardWhy(issue) === ""
                        readonly property bool canConfirm: root.confirmWhy(issue) === ""
                        readonly property bool canReopen: root.reopenWhy(issue) === ""
                        Keys.onEscapePressed: root.closeIssue()
                        property bool hasNote: note.text.trim() !== ""   // the core rejects these without one
                        readonly property int etaHours: parseInt(eta.text) || 0
                        function act(a, eta) {
                            root.run("act", [issue.id, a, note.text, nextStep.text, eta | 0],
                                     function () { note.text = ""; nextStep.text = "" })
                        }
                        ColumnLayout {
                            x: (detail.availableWidth - width) / 2
                            width: Math.min(detail.availableWidth, 760)
                            spacing: 0
                            RowLayout {
                                Layout.fillWidth: true; Layout.bottomMargin: 8
                                Item { visible: !root.narrow; Layout.fillWidth: true }
                                FramedButton { text: root.narrow ? "← Board" : "Close"; onClicked: root.closeIssue() }
                            }
                            Flow {   // code, stage, reopens, the reporter's call to action, category
                                Layout.fillWidth: true; Layout.bottomMargin: 4
                                spacing: 6
                                Label { height: 24; verticalAlignment: Text.AlignVCenter; textFormat: Text.RichText; text: root.mono(root.placeCode(detail.issue)) }
                                StatusChip { label: detail.issue.stage || ""; stage: detail.stage; icon: root.stageIcon(detail.st); maxWidth: parent.width }
                                StatusChip {
                                    visible: detail.issue.reopen_count > 0; maxWidth: parent.width
                                    label: "Reopened " + detail.issue.reopen_count + "×"; stage: root.t.closed; icon: "reopen"
                                }
                                StatusChip {
                                    visible: detail.mine && detail.canConfirm; maxWidth: parent.width; strong: true
                                    label: "Needs your confirmation"; icon: "bell"
                                    stage: ({ fg: root.t.onPrimary, bg: root.t.primary, bd: root.t.primary })
                                }
                                Label { height: 24; verticalAlignment: Text.AlignVCenter; text: "· " + (detail.issue.category || ""); font.pointSize: root.smallSize; color: root.t.muted }
                            }
                            Label {
                                Layout.fillWidth: true; Layout.topMargin: 4; wrapMode: Text.Wrap
                                text: !detail.issue.location ? "" : root.placeName(detail.issue) || root.placeCode(detail.issue)
                                font.pointSize: root.baseSize * 17 / 14; font.weight: Font.DemiBold
                            }
                            Label {
                                visible: !!detail.issue.location_renamed_from
                                Layout.fillWidth: true; wrapMode: Text.Wrap
                                text: "Place renamed from " + detail.issue.location_renamed_from; font.pointSize: root.smallSize; color: root.t.muted
                            }
                            Rectangle {   // the report itself, as the reporter wrote it
                                Layout.fillWidth: true; Layout.topMargin: 6; Layout.bottomMargin: 14
                                implicitHeight: quote.implicitHeight + 2
                                radius: 6; color: root.t.page; border.color: root.t.bd
                                ColumnLayout {
                                    id: quote
                                    x: 1; y: 1; width: parent.width - 2
                                    spacing: 0
                                    Label {
                                        Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.RichText
                                        leftPadding: 12; rightPadding: 12; topPadding: 6; bottomPadding: 6
                                        font.pointSize: root.smallSize; color: root.t.muted
                                        text: "Reported by " + root.esc(root.who(detail.issue.reporter, detail.issue.reporter_name)) + " "
                                              + root.mono(String(detail.issue.reporter).substr(0, 6)) + " · " + root.when(detail.issue.reported_ts)
                                        background: Rectangle {
                                            color: root.t.well; topLeftRadius: 5; topRightRadius: 5
                                            Rectangle { width: parent.width; height: 1; anchors.bottom: parent.bottom; color: root.t.bd }
                                        }
                                    }
                                    Label {
                                        Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.PlainText
                                        leftPadding: 12; rightPadding: 12; topPadding: 8; bottomPadding: 8
                                        text: detail.issue.text || ""; font.pointSize: root.baseSize * 16 / 14
                                    }
                                }
                            }
                            WhyCard {
                                readonly property var plan: root.whyCard(detail.issue)
                                visible: plan !== null
                                Layout.fillWidth: true; Layout.bottomMargin: 16
                                card: plan || ({ intro: "", steps: [], never: "" }); stage: detail.stage
                            }
                            Label { Layout.bottomMargin: 8; text: "Timeline"; font.pointSize: root.smallSize; font.weight: Font.DemiBold; color: root.t.muted }
                            Item {   // the events on one spine
                                Layout.fillWidth: true; Layout.bottomMargin: 16
                                implicitHeight: events.implicitHeight
                                Rectangle { x: 13; y: 6; width: 2; height: Math.max(0, parent.height - 12); color: root.t.bd }
                                ColumnLayout {
                                    id: events
                                    width: parent.width
                                    spacing: 12
                                    Repeater {
                                        model: root.selected ? root.selected.events : []
                                        delegate: TimelineRow {
                                            required property var modelData
                                            Layout.fillWidth: true
                                            event: modelData
                                            byReporter: modelData.author === detail.issue.reporter
                                        }
                                    }
                                }
                            }
                            Rectangle { Layout.fillWidth: true; Layout.bottomMargin: 14; implicitHeight: 1; color: root.t.bd }
                            FormRow {
                                label: "Note"; sub: "(what was seen, done or why)"
                                FramedField { id: note; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Note"; placeholderText: "e.g. Pump checked, handle is loose"; onAccepted: comment.submit() }
                            }
                            RowLayout {
                                visible: detail.steward
                                Layout.topMargin: 10
                                spacing: 8
                                FormRow {
                                    label: "Next step"; sub: "(for updates)"
                                    FramedField { id: nextStep; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Next step"; placeholderText: "e.g. Plumber visits tomorrow" }
                                }
                                FormRow {
                                    label: "ETA (hours)"; strong: true
                                    Layout.fillWidth: false
                                    FramedField {
                                        id: eta
                                        Layout.preferredWidth: 96
                                        text: "4"
                                        placeholderText: "e.g. 4"
                                        inputMethodHints: Qt.ImhDigitsOnly
                                        validator: IntValidator { bottom: 0; top: 168 }
                                        Accessible.name: "ETA in hours"
                                    }
                                }
                            }
                            ColumnLayout {
                                Layout.fillWidth: true; Layout.topMargin: 6
                                enabled: !root.syncing
                                spacing: 6
                                ActionGroup {
                                    title: "For stewards"
                                    visible: detail.steward
                                    ActionButton { visible: detail.st === "Open"; kind: "primary"; text: "Acknowledge"; onClicked: detail.act("acknowledge", detail.etaHours) }
                                    ActionButton { allowed: detail.hasNote; text: "Post update"; onClicked: detail.act("update", detail.etaHours) }
                                    ActionButton {
                                        allowed: detail.hasNote; kind: detail.st === "Open" ? "default" : "primary"
                                        text: "Claim fixed (say what was done)"; onClicked: detail.act("claim_resolved", 0)
                                    }
                                    ActionButton { allowed: detail.hasNote; kind: "danger"; text: "Won't fix (reason)"; onClicked: detail.act("close_wontfix", 0) }
                                    RowLayout {   // close this one as a duplicate of another open issue
                                        width: parent.width
                                        spacing: 8
                                        FramedCombo {
                                            // The model is rebuilt on every 2 s refresh, which resets currentIndex,
                                            // so the user's pick is kept in `target`, never read from currentIndex.
                                            id: dupOf
                                            Accessible.name: "Duplicate of another report"
                                            property string target: ""
                                            property string targetLabel: ""
                                            property string forIssue: detail.issue.id || ""
                                            onForIssueChanged: target = ""
                                            Layout.fillWidth: true
                                            Layout.preferredWidth: 260
                                            textRole: "label"
                                            valueRole: "id"
                                            displayText: target ? targetLabel : "Duplicate of…"
                                            model: root.duplicateTargets(detail.issue.id)
                                            onActivated: function (index) { target = valueAt(index); targetLabel = textAt(index) }
                                        }
                                        ActionButton {
                                            text: "Mark duplicate"
                                            allowed: dupOf.target !== ""
                                            onClicked: root.run("act", [detail.issue.id, "mark_duplicate", dupOf.target, "", 0],
                                                                function () { dupOf.target = "" })
                                        }
                                    }
                                }
                                ActionGroup {
                                    // the reporter decides alone; another resident's confirm or reopen is one of two votes
                                    title: detail.mine ? "For the person who reported it" : "For residents"
                                    visible: detail.canConfirm || detail.canReopen
                                    ActionButton {
                                        visible: detail.canConfirm; kind: "primary"
                                        text: detail.mine ? "Confirm it's fixed" : "Confirm (" + detail.issue.confirms + " of 2 residents so far)"
                                        onClicked: detail.act("confirm", 0)
                                    }
                                    ActionButton {
                                        visible: detail.canReopen
                                        kind: detail.canConfirm ? "default" : "primary"
                                        allowed: detail.hasNote
                                        text: "Reopen (reason)" + (detail.issue.reopen_count > 0 ? ", reopened " + detail.issue.reopen_count + "×" : "")
                                        onClicked: detail.act("reopen", 0)
                                    }
                                }
                                ActionGroup {
                                    title: "For everyone"
                                    ActionButton { id: comment; allowed: detail.hasNote; text: "Comment"; onClicked: detail.act("comment", 0) }
                                }
                            }
                            Rectangle {   // what this viewer can't do here, and why (the same rules as above)
                                id: na
                                readonly property var lines: root.unavailable(detail.issue, detail.hasNote)
                                visible: lines.length > 0
                                Layout.fillWidth: true; Layout.topMargin: 14; Layout.bottomMargin: 16
                                implicitHeight: naCol.implicitHeight + 20
                                radius: 6; color: root.t.well; border.color: root.t.bd
                                Column {
                                    id: naCol
                                    x: 12; y: 10; width: parent.width - 24
                                    spacing: 2
                                    Label { text: "Not available to you:"; font.pointSize: root.smallSize; font.weight: Font.DemiBold }
                                    Repeater {
                                        model: na.lines
                                        delegate: Label {
                                            required property string modelData
                                            width: naCol.width; wrapMode: Text.Wrap
                                            text: modelData; font.pointSize: root.smallSize; color: root.t.muted
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Report: two steps, "What's wrong?" and "Where?" (one grouped, filterable place list)
            TabPage {
                id: reportPage
                enabled: !root.syncing
                property string pick: ""          // "" = nothing yet, "other", or a place code (never a retired one)
                property var retired: null        // the retired place last clicked: its notice shows until another pick
                function choose(p) { pick = p; retired = null }
                // What still blocks the Report button, in words ("" = ready).
                readonly property string missing: reportText.text.trim() === "" ? "Describe the problem in one line."
                    : pick === "" ? "Pick where it is."
                    : pick === "other" && landmark.text.trim() === "" ? "Describe the place." : ""
                Note {
                    visible: !root.approved
                    font.bold: true
                    text: "Waiting for the admin to approve you. Read your fingerprint aloud at the kiosk; you can report once you are approved."
                }
                Heading { text: "What's wrong?"; Layout.topMargin: 0 }
                FormRow {
                    label: "Category"
                    FramedCombo { id: category; Accessible.name: "What is wrong (category)"; model: root.categories; Layout.preferredWidth: 260 }
                }
                // maximumLength counts characters; the core's 500 limit is bytes (Hindi is 3 B/char),
                // so a long non-Latin line comes back as "error: too long" instead.
                FormRow {
                    label: "In one line"
                    FramedField {
                        id: reportText; Layout.fillWidth: true; maximumLength: 500
                        placeholderText: "e.g. Tap near tent 4 gives no water"
                        Accessible.name: "What is wrong, one line"; onAccepted: reportBtn.submit()
                    }
                }
                Heading { text: "Where?" }
                PlaceBrowser {
                    maxHeight: 220
                    retiredPlace: reportPage.retired
                    onReportOther: {      // keep category and description; the landmark starts as an editable "near ..."
                        landmark.text = "near " + reportPage.retired.code + " " + reportPage.retired.label
                        reportPage.choose("other")
                        landmark.forceActiveFocus()
                    }
                    listHeader: Component {
                        PlaceChoice {
                            text: "Other (describe the place)"
                            picked: reportPage.pick === "other"
                            onClicked: reportPage.choose("other")
                        }
                    }
                    rowDelegate: Component {
                        PlaceChoice {
                            required property var modelData
                            width: ListView.view.width
                            text: modelData.code + "  " + modelData.label + (modelData.state !== "active" ? "  (" + root.placeState(modelData) + ")" : "")
                            retired: modelData.state !== "active"
                            explains: true
                            picked: reportPage.pick === modelData.code
                            onClicked: modelData.state !== "active" ? reportPage.retired = modelData : reportPage.choose(modelData.code)
                        }
                    }
                }
                Note {
                    color: root.t.muted
                    text: reportPage.pick === "" ? "No place picked yet."
                        : reportPage.pick === "other" ? "Other place: describe it below."
                        : "Picked: " + root.placeLabel(reportPage.pick)
                }
                FormRow {
                    visible: reportPage.pick !== ""
                    label: reportPage.pick === "other" ? "Describe the place (required)" : "Landmark (optional)"
                    FramedField {
                        id: landmark; Layout.fillWidth: true; maximumLength: 500
                        Accessible.name: reportPage.pick === "other" ? "Describe the place" : "Landmark"
                        placeholderText: reportPage.pick === "other" ? "e.g. pipe behind tent 4, by the neem tree" : "e.g. the left tap, behind the water tank"
                    }
                }
                ActionButton {
                    id: reportBtn
                    text: "Report"
                    allowed: root.approved && reportPage.missing === ""
                    onClicked: root.run("report", [category.currentText, reportPage.pick, landmark.text, reportText.text],
                                        function () { reportText.text = ""; landmark.text = ""; reportPage.pick = ""; tabs.currentIndex = 0 })
                }
                Note { visible: root.approved && reportPage.missing !== ""; color: root.t.muted; text: reportPage.missing }
                Heading { visible: !root.staff; text: "Your name" }
                Note {
                    visible: !root.staff
                    text: "Leave it empty to stay a pseudonym; the admin who granted your role can still link it. Staff are always named."
                }
                LabelledField {
                    id: nameField
                    visible: !root.staff
                    label: "Display name"
                    placeholder: "e.g. Asha (or leave empty to stay pseudonymous)"
                    buttonText: "Set name"
                    onSubmitted: root.run("set_profile", [nameField.text])
                }
            }

            // Members (admin): grant pending keys after the fingerprint is read aloud
            TabPage {
                id: membersPage
                enabled: !root.syncing
                readonly property var renamed: root.locations.filter(function (l) { return !!l.renamed_from })
                Heading { text: "Waiting for approval"; Layout.topMargin: 0 }
                Note { text: "Grant a role only after the person reads this fingerprint aloud." }
                Note { visible: root.pending.length === 0; color: root.t.muted; text: "No one is waiting." }
                Repeater {
                    model: root.pending
                    delegate: RowLayout {
                        id: pendingRow
                        Layout.fillWidth: true
                        spacing: 8
                        required property var modelData
                        Label { Layout.fillWidth: true; elide: Text.ElideRight; text: pendingRow.modelData.fingerprint + "  " + (pendingRow.modelData.name || "(no name)"); font.family: "monospace" }
                        ActionButton { text: "Grant resident"; onClicked: root.run("grant_role", [pendingRow.modelData.key, "resident", ""]) }
                        LabelledField {
                            id: staffName
                            Layout.fillWidth: false
                            Layout.preferredWidth: 300
                            name: "Steward's real name"
                            placeholder: "e.g. Ravi Kumar"
                            buttonText: "Grant steward"
                            buttonEnabled: staffName.text.trim().length > 0
                            onSubmitted: root.run("grant_role", [pendingRow.modelData.key, "steward", staffName.text])
                        }
                    }
                }
                Heading { text: "Members" }
                Repeater {
                    model: root.members
                    delegate: LabelledField {
                        id: memberRow
                        required property var modelData
                        readonly property bool own: modelData.key === root.me.key
                        caption: modelData.fingerprint + "  " + modelData.role + "  " + (modelData.name || "pseudonym")
                        name: "Reason for revoking " + modelData.fingerprint
                        placeholder: own ? "you can't revoke yourself" : "e.g. left the camp"
                        fieldEnabled: !own
                        buttonText: "Revoke"
                        buttonEnabled: !own && memberRow.text.trim().length > 0
                        onSubmitted: root.run("revoke_role", [modelData.key, memberRow.text])
                    }
                }
                Heading { text: "Add a location" }
                Note { text: "For a place found on the site walk. It joins the list straight away." }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    FormRow {
                        label: "Group"
                        Layout.fillWidth: false
                        FramedCombo {
                            id: locGroup
                            Layout.preferredWidth: 240
                            Accessible.name: "Group for the new location"
                            model: root.groupNames.concat(["New group…"])
                            onActivated: locNewGroup.text = ""     // each "New group…" starts blank
                        }
                    }
                    FormRow {
                        visible: locGroup.currentIndex === root.groupNames.length
                        label: "New group name"
                        FramedField { id: locNewGroup; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "New group name"; placeholderText: "e.g. Kitchens" }
                    }
                }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    FormRow {
                        label: "Code"
                        Layout.fillWidth: false
                        FramedField { id: locCode; Layout.preferredWidth: 120; maximumLength: 500; Accessible.name: "Code"; placeholderText: "e.g. W-04" }
                    }
                    FormRow {
                        label: "Name"
                        FramedField { id: locLabel; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Name"; placeholderText: "e.g. Tap behind tent 4"; onAccepted: addLoc.submit() }
                    }
                }
                ActionButton {
                    id: addLoc
                    text: "Add location"
                    readonly property string group: locGroup.currentIndex === root.groupNames.length ? locNewGroup.text.trim() : locGroup.currentText
                    allowed: locCode.text.trim() !== "" && locLabel.text.trim() !== "" && group !== ""
                    onClicked: root.run("add_location", [locCode.text, locLabel.text, group],
                                        function () { locCode.text = ""; locLabel.text = ""; locNewGroup.text = "" })
                }
                Heading { text: "Locations" }
                Note {
                    text: "Nothing is deleted. A retired place takes no new reports; one with open issues can't be retired. "
                        + "A place nobody ever reported can be removed: it is hidden after 30 days (undo until then), and its signed events stay in everyone's log. "
                        + "Edit changes a place's name and group; its code never changes."
                }
                PlaceBrowser {
                    maxHeight: 420
                    rowDelegate: Component {
                        ColumnLayout {
                            id: locRow
                            required property var modelData
                            readonly property string st: modelData.state
                            readonly property bool pending: st === "pending_removal"
                            // The core refuses a retire while issues are open; disabling is only a courtesy.
                            readonly property bool blocked: st === "active" && modelData.open_issues > 0
                            readonly property bool hasReason: reason.text.trim() !== ""
                            property bool editing: false
                            function change(method) { root.run(method, [modelData.code, reason.text], function () { reason.text = "" }) }
                            width: ListView.view.width
                            spacing: 4
                            RowLayout {
                                Layout.fillWidth: true
                                Layout.leftMargin: 8; Layout.rightMargin: 8
                                spacing: 8
                                Label {
                                    Layout.fillWidth: true
                                    elide: Text.ElideRight
                                    font.family: "monospace"; textFormat: Text.PlainText
                                    color: locRow.st === "active" ? root.t.fg : root.t.muted
                                    text: locRow.modelData.code + "  " + locRow.modelData.label
                                        + (locRow.st === "retired" ? "  (retired: " + locRow.modelData.retired_reason + ")" : "")
                                        + (locRow.pending ? "  (" + root.placeState(locRow.modelData) + ": " + locRow.modelData.removal_reason + ")" : "")
                                        + (locRow.blocked ? "  · " + root.openIssuesText(locRow.modelData.open_issues) : "")
                                }
                                FramedField {
                                    id: reason
                                    Layout.preferredWidth: 170
                                    maximumLength: 500
                                    enabled: !locRow.blocked
                                    placeholderText: locRow.blocked ? "close its issues first" : "e.g. duplicate"
                                    Accessible.name: "Reason for changing " + locRow.modelData.code
                                }
                                RowLayout {   // fixed width, right-aligned: every row's reason box and Edit line up
                                    Layout.preferredWidth: 84 + 108 + 84 + 3 * 8     // three slots and the spacer, 8 apart
                                    Layout.fillWidth: false
                                    spacing: 8
                                    Item { Layout.fillWidth: true }
                                    ActionButton {   // slot 1: Retire / Restore
                                        shown: !locRow.pending
                                        Layout.preferredWidth: 84
                                        text: locRow.st === "retired" ? "Restore" : "Retire"
                                        allowed: locRow.hasReason && !locRow.blocked
                                        onClicked: locRow.change(locRow.st === "retired" ? "restore_location" : "retire_location")
                                    }
                                    ActionButton {   // slot 2: Remove / Undo removal; only for a place no report ever named, the core decides
                                        shown: locRow.pending || !locRow.modelData.ever_used
                                        Layout.preferredWidth: 108
                                        text: locRow.pending ? "Undo removal" : "Remove"
                                        allowed: locRow.hasReason
                                        onClicked: locRow.change(locRow.pending ? "undo_remove_location" : "remove_location")
                                    }
                                    ActionButton {   // slot 3: Edit
                                        shown: !locRow.pending
                                        Layout.preferredWidth: 84
                                        text: locRow.editing ? "Cancel" : "Edit"
                                        Accessible.name: (locRow.editing ? "Cancel editing " : "Edit ") + locRow.modelData.code
                                        onClicked: locRow.editing = !locRow.editing
                                    }
                                }
                            }
                            RowLayout {   // the code is the place's identity: only name and group change
                                visible: locRow.editing
                                Layout.fillWidth: true
                                Layout.leftMargin: 8; Layout.rightMargin: 8; Layout.bottomMargin: 6
                                spacing: 8
                                FormRow {
                                    label: "Name of " + locRow.modelData.code
                                    FramedField { id: editLabel; Layout.fillWidth: true; maximumLength: 500; text: locRow.modelData.label; Accessible.name: "New name for " + locRow.modelData.code; onAccepted: save.submit() }
                                }
                                FormRow {
                                    label: "Group"
                                    Layout.fillWidth: false
                                    FramedField { id: editGroup; Layout.preferredWidth: 200; maximumLength: 500; text: locRow.modelData.group; Accessible.name: "New group for " + locRow.modelData.code; onAccepted: save.submit() }
                                }
                                ActionButton {
                                    id: save
                                    Layout.alignment: Qt.AlignBottom
                                    text: "Save"
                                    allowed: editLabel.text.trim() !== "" && editGroup.text.trim() !== ""
                                    onClicked: root.run("edit_location", [locRow.modelData.code, editLabel.text, editGroup.text],
                                                        function () { locRow.editing = false })
                                }
                            }
                        }
                    }
                }
                Heading { text: "Change log" }
                Note {
                    color: root.t.muted
                    text: "Removed places are hidden from every list, retired ones included. Removed means hidden: the signed events stay in everyone's log."
                }
                Note {
                    visible: root.removedLocations.length === 0 && membersPage.renamed.length === 0
                    color: root.t.muted
                    text: "Nothing removed or renamed yet."
                }
                Repeater {
                    model: root.removedLocations
                    delegate: Note {
                        required property var modelData
                        text: modelData.code + "  " + modelData.label + " (" + modelData.group + "): removed " + root.when(modelData.removed_at)
                            + ". " + root.who(modelData.by, modelData.by_name) + " started the removal " + root.when(modelData.since)
                            + ", reason: " + modelData.reason
                    }
                }
                Repeater {
                    model: membersPage.renamed
                    delegate: Note {
                        required property var modelData
                        text: modelData.code + "  renamed from \u201c" + modelData.renamed_from + "\u201d to \u201c" + modelData.label + "\u201d"
                    }
                }
            }

            // Anchor: compute, run the printed spel command in a terminal, record the reference
            TabPage {
                enabled: !root.syncing
                Note {
                    text: "An anchor writes a fingerprint of everyone's history to the Logos blockchain (LEZ), so no one, "
                        + "not even the admin, can quietly rewrite or delete past events. Anyone can recompute it from their own copy and compare."
                }
                Heading { text: "1. Compute the checkpoint" }
                ActionButton { text: "Compute checkpoint"; onClicked: root.run("checkpoint_now", [], function (r) { if (r && r.heads_root) root.checkpoint = r }) }
                Label { visible: !!root.checkpoint; text: root.checkpoint ? root.checkpoint.n_events + " events, root " + root.checkpoint.heads_root.substr(0, 16) + "…" : "" }
                Heading { text: "2. Run the command" }
                Note { text: "Copy this command and run it from programs/pukaar_registry/ in a terminal. Replace <YOUR_PUBLIC_ACCOUNT> with your LEZ account." }
                FramedTextArea {
                    readOnly: true
                    name: "spel anchor command"
                    placeholder: "Compute the checkpoint first"
                    text: root.checkpoint ? root.checkpoint.spel : ""
                }
                Heading { text: "3. Record the anchor" }
                Note { text: "Paste the tx hash or pda:<id> that spel prints, then record it." }
                LabelledField {
                    id: anchorField
                    label: "Anchor reference"
                    placeholder: "e.g. 0x9f2c…e1 or pda:<account id> printed by spel"
                    buttonText: "Record anchor"
                    buttonEnabled: !!root.checkpoint && anchorField.text.trim() !== ""
                    onSubmitted: {
                        root.run("record_anchor", [JSON.stringify(root.checkpoint.heads), anchorField.text],
                                 function () { anchorField.text = ""; root.checkpoint = null })
                    }
                }
            }

            // Identity: a password-sealed backup, to continue as the same person after a reinstall
            TabPage {
                Heading { text: "Back up your identity"; Layout.topMargin: 0 }
                Note {
                    text: "Keep this and your password safe. Anyone with both can act as you. Never run the same identity on two devices at once."
                    color: root.t.danger
                }
                FormRow {
                    label: "Password (at least 8 characters)"
                    FramedField { id: exportPassword; Layout.fillWidth: true; echoMode: TextInput.Password; Accessible.name: "Backup password"; placeholderText: "e.g. a long phrase only you know" }
                }
                LabelledField {
                    id: exportConfirm
                    label: "Repeat the password"
                    placeholder: "type the same password again"
                    echoMode: TextInput.Password
                    buttonText: "Export identity"
                    buttonEnabled: exportPassword.text.length >= 8 && exportConfirm.text === exportPassword.text
                    onSubmitted: {
                        root.run("export_identity", [exportPassword.text],
                                 function (r) { exportOut.text = r; exportPassword.text = ""; exportConfirm.text = "" })
                    }
                }
                FramedTextArea {
                    id: exportOut
                    visible: text !== ""
                    readOnly: true
                    name: "identity backup"
                }
                FramedButton { visible: exportOut.text !== ""; text: "Hide"; onClicked: exportOut.text = "" }
            }
        }
    }
}
