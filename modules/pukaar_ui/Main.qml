// Pukaar view. QML-only: every call goes to pukaar_core through the host's
// `logos` bridge, and the view re-reads state every 2 s (no module events at L1).
pragma ComponentBehavior: Bound
import QtCore
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
    // rebuilt model would wipe typed text in the People rows and reset the Report combos.
    property var categories: []
    property var locations: []
    property var placeLog: []            // the admin's change log (site_info place_log), newest first
    property var members: []
    property var pending: []
    property var revoked: []             // members whose role the admin removed (site_info revoked), newest first
    property var groupNames: []             // place groups in first-seen order; written only when they change
    property var issues: []
    property var selected: null          // issue_timeline() result
    property var checkpoint: null        // checkpoint_now() result
    property string message: ""
    property bool onlyMine: false
    property string historyPlace: ""     // History's place filter: a place code, "" = every place
    property string notice: ""           // an info banner after an action: where a reopened issue went
    property bool busy: false            // a call is running: every action button waits (no double submit)
    readonly property bool inSite: !!(me && me.site)
    readonly property bool staff: me.role === "Steward" || me.role === "Admin"
    readonly property bool approved: !!me.role      // a pending key has no role yet; the core rejects its reports
    // Our role was revoked, and we have not asked to join again (a new profile puts us back in pending).
    readonly property bool removed: inSite && !approved && !!me.revoked && !pending.some(function (p) { return p.key === me.key })
    readonly property bool unapproved: !approved && !removed      // no role, and it is ours to ask for: read the fingerprint at the kiosk
    readonly property bool awaitingApproval: inSite && unapproved && !!info.name   // joined, genesis here, not yet granted a role
    readonly property bool isAdmin: me.role === "Admin"
    readonly property bool isSuper: !!me.super_admin     // changes admins; at most two per site
    readonly property int superCount: (info.super_admins || []).length
    readonly property string roleName: isSuper ? "Super admin" : me.role || (removed ? "Removed" : "Pending")
    // A restored identity waits for its own history before signing anything (core refuses meanwhile).
    readonly property bool syncing: !!me.syncing_own_history
    // ---- shell (design/proposal/NOTES.md section 3) ----
    readonly property var pages: ["board", "report", "people", "places", "proof", "profile", "history"]   // the StackLayout's children, in order
    property string page: "board"
    // The sidebar is expanded (216 px) from 1100 px of window, an 80 px icon rail below; the user's
    // toggle ("expanded" / "collapsed", "" = never pressed) wins at every width and is remembered.
    // Below 930 px the rail never grows (four 160 px columns would not fit): Expand opens the
    // full sidebar over the board for now (overlayOpen, not remembered).
    Settings { id: prefs; category: "pukaar"; property string sidebar: "" }
    readonly property bool overlay: width < 930
    property bool overlayOpen: false
    onOverlayChanged: overlayOpen = false
    readonly property bool sidebarOpen: overlay ? overlayOpen : prefs.sidebar !== "" ? prefs.sidebar === "expanded" : width >= 1100
    readonly property int sidebarWidth: sidebarOpen && !overlay ? 216 : 80     // what the content gives up
    function toggleSidebar() { if (overlay) overlayOpen = !overlayOpen; else prefs.sidebar = sidebarOpen ? "collapsed" : "expanded" }
    function openPage(name) { page = name; overlayOpen = false }
    // A place code's link: that place in History. Narrow, the pane covers the page, so it closes.
    function showPlace(code) { historyPlace = code; if (narrow) closeIssue(); openPage("history") }
    // The issue pane sits beside the Board and History pages (both open issues); narrow, it replaces them.
    readonly property bool paneOpen: selected !== null && (page === "board" || page === "history")
    // Four board columns (160 each, 12 apart) + the 400 px pane, its 8 px gap and the content's 16 px
    // margins need 1116 px of content; below that the pane replaces the board instead of squeezing it.
    readonly property int columnsWidth: 4 * 160 + 3 * 12
    readonly property bool narrow: width - sidebarWidth < columnsWidth + 8 + 400 + 2 * 16

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
        warn: "#9a6700", warnBg: "#fff8c5", warnBd: "#66d4a72c",
        okDot: "#1a7f37", connDot: "#9a6700",
        reported: { fg: "#bc4c00", bg: "#fff1e5", bd: "#99fb8f44", edge: "#bc4c00" },
        progress: { fg: "#0969da", bg: "#ddf4ff", bd: "#9954aeff", edge: "#0969da" },
        awaiting: { fg: "#8250df", bg: "#fbefff", bd: "#99c297ff", edge: "#8250df" },
        resolved: { fg: "#1a7f37", bg: "#dafbe1", bd: "#994ac26b", edge: "#1a7f37" },
        closed: { fg: "#59636e", bg: "#eff1f3", bd: "#d1d9e0", edge: "#818b98" },
        sb: "#f6f8fa", sbCard: "#ffffff", sbHover: "#eaeef2", sbFg: "#1f2328", sbMuted: "#59636e", sbIcon: "#59636e",
        sbEdge: "#d1d9e0", sbLine: "#d1d9e0", sbCtl: "#818b98", sbActBg: "#c8e6ff", sbActFg: "#0550ae", sbActIcon: "#0550ae",
        sbAv: "#dde3ea", sbOk: "#1a7f37", sbConn: "#9a6700", sbOff: "#d1242f", sbDanger: "#d1242f",
        tipBg: "#25292e", tipFg: "#ffffff", tipBd: "#25292e"
    })
    readonly property var darkTokens: ({
        fg: "#ffffff", muted: "#a4a4a4", ph: "#8c92a0", disabled: "#7a7a7a",
        page: "#171717", well: "#1c1c1c", card: "#232323", field: "#262626", hover: "#2b2b2b", btn: "#2b2b2b", btnOff: "#1f1f1f",
        bd: "#434343", ctlBd: "#808080", btnBd: "#4d4d4d",
        accent: "#6aa8f0", accentBg: "#1f2d3d", accentBd: "#4a90e2", focus: "#6aa8f0",
        primary: "#2563c9", primaryHover: "#2f6fd0", onPrimary: "#ffffff",
        danger: "#ff8a82", dangerBg: "#2e1f1f", dangerBd: "#fb3748",
        warn: "#febc2e", warnBg: "#2e2914", warnBd: "#bb8009",
        okDot: "#49f563", connDot: "#febc2e",
        reported: { fg: "#ff9a2e", bg: "#3b2814", bd: "#ff8800", edge: "#ff8800" },
        progress: { fg: "#6aa8f0", bg: "#1f2d3d", bd: "#4a90e2", edge: "#4a90e2" },
        awaiting: { fg: "#b794f6", bg: "#2f2840", bd: "#a07bea", edge: "#a07bea" },
        resolved: { fg: "#6ccc93", bg: "#22362b", bd: "#6ccc93", edge: "#6ccc93" },
        closed: { fg: "#b5b5b5", bg: "#2f2f2f", bd: "#808080", edge: "#808080" },
        sb: "#141414", sbCard: "#1c1c1c", sbHover: "#262626", sbFg: "#ebebeb", sbMuted: "#a4a4a4", sbIcon: "#a4a4a4",
        sbEdge: "#2c2c2c", sbLine: "#343434", sbCtl: "#808080", sbActBg: "#243b55", sbActFg: "#ffffff", sbActIcon: "#6aa8f0",
        sbAv: "#2b303b", sbOk: "#49f563", sbConn: "#febc2e", sbOff: "#ff736a", sbDanger: "#ff8a82",
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
    // Hindi in user text needs no code here: Qt falls back per glyph to any system Devanagari font
    // (Kohinoor on macOS, Noto Sans Devanagari / Lohit on Linux). A kiosk with none installed needs
    // the fonts-noto-core package: the module builder ships only Main.qml and the icon, so a font
    // file cannot travel inside the .lgx.
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
    function isTerminal(status) { return ["ConfirmedResolved", "ClosedWontfix", "Duplicate"].indexOf(status) >= 0 }

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
    // failures ({"error": …}: the core isn't loaded, or didn't answer within the bridge's 20 s)
    // are folded into the "error: …" strings the core uses, in words, with the raw text kept last.
    // A timed-out call may still finish in the core, hence "if it went through".
    function decode(v) {
        try { v = JSON.parse(v) } catch (e) {}
        if (typeof v === "string") { try { v = JSON.parse(v) } catch (e) {} }
        if (v && typeof v.error === "string")
            return "error: Pukaar's core did not answer. If it went through after all, this screen updates by itself; "
                    + "otherwise try again in a moment (what you typed is kept). Details: "
                    + v.error + (v.message ? ": " + v.message : "")
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
        keep("categories", i.categories); keep("locations", i.locations); keep("placeLog", i.place_log || [])
        keep("members", i.members); keep("pending", i.pending); keep("revoked", i.revoked || [])
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
        notice = ""
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
            closeIssue(); page = "board"; checkpoint = null; issues = []; setInfo(noInfo); seen = ({})
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
    // ---- the board's view rule (NOTES.md section 8): nothing is removed from `issues`. Resolved
    // cards stay 14 days after their closing status change and Closed ones 30, newest-closed first,
    // at most boardCap before "+N more"; older ones are in History. Live work always shows. ----
    // ponytail: a fixed cap, not a setting. Ceiling: a 100+ issue fortnight still gives a long column
    // once expanded, whatever the screen; upgrade path: a Genesis rule, or paging the expanded rows.
    readonly property int boardCap: 12
    // The core rewrites progress on every status change and never on a comment, so for a finished
    // issue progress.ts is its closing time; a reopen restarts the clock.
    function closedAt(i) { return i.progress ? i.progress.ts : i.reported_ts }
    function windowDays(status) { return status === "ConfirmedResolved" ? 14 : 30 }
    function onBoard(i) { return !isTerminal(i.status) || Date.now() / 1000 - closedAt(i) < windowDays(i.status) * 86400 }
    function newestClosedFirst(a, b) { return closedAt(b) - closedAt(a) }
    function mineOk(i) { return !onlyMine || i.reporter === me.key }
    // A column's issues: `board` true = what the window keeps, false = the older ones (History only).
    function inColumn(statuses, board) {
        var l = issues.filter(function (i) { return statuses.indexOf(i.status) >= 0 && mineOk(i) && onBoard(i) === board })
        return isTerminal(statuses[0]) ? l.sort(newestClosedFirst) : l
    }
    readonly property int boardCount: issues.filter(mineOk).length
    // Clipboard via a hidden TextEdit (QML has no direct clipboard API); the sidebar Copy and FramedTextArea share it.
    TextEdit { id: clip; visible: false }
    function copyText(t) { clip.text = t; clip.selectAll(); clip.copy() }
    function initials(name) { return String(name || "").split(/[\s\-_]+/).filter(Boolean).slice(0, 2).map(function (w) { return w[0] }).join("").toUpperCase() }
    function shortId(hex) { return String(hex).substr(0, 8) + "…" + String(hex).substr(-4) }
    function who(key, name) { return name ? name : "pseudonym " + String(key).substr(0, 6) }
    function formerTag(former) { return former ? " (former member)" : "" }   // after a revoked author's name and code
    function when(ts, format) { return new Date(ts * 1000).toLocaleString(Qt.locale(), format || "d MMM HH:mm") }
    // A member's admin notice (site_info notices): the running one while they are an admin, else the
    // one that ended it, or their last demotion.
    function noticeOf(m) {
        var n = (info.notices || []).filter(function (x) { return x.subject === m.key })[0], d = m.demotions || []
        return m.role === "Admin" ? n || null : n || d[d.length - 1] || null
    }
    // The People row's line about it.
    function noticeLine(m) {
        var n = noticeOf(m)
        if (!n) return ""
        var by = who(n.by, n.by_name), why = "“" + n.reason + "”"
        if (m.role === "Admin")
            return "Admin · " + (n.outcome === "remove" ? "is removed" : "becomes steward") + " on " + when(n.deadline) + " (notice by " + by + ": " + why + ")"
        return m.role + " · was admin until " + when(n.deadline) + ", changed by " + by + ": " + why
               + (n.sealed_by ? " Sealed by " + who(n.sealed_by, n.sealed_by_name) + (n.sealed_ts ? " at " + when(n.sealed_ts) : "") + "." : "")
    }
    // The member's own refused role actions (site_info rejected), e.g. an admin's after their notice ended.
    function rejectedLine(m) {
        var l = (info.rejected || []).filter(function (r) { return r.author === m.key }), n = noticeOf(m)
        if (!l.length) return ""
        return plural(l.length, "action") + " by " + who(m.key, m.name) + (n && m.role !== "Admin" ? " after " + when(n.deadline) : "")
               + (l.length === 1 ? " was" : " were") + " rejected: " + l[0].reason
    }
    // The change log's action chip per kind: its label and colour group (blue in motion, green restored, grey ended).
    readonly property var placeActions: ({ added: ["Added", "closed"], edited: ["Edited", "progress"], renamed: ["Renamed", "progress"],
        retired: ["Retired", "closed"], restored: ["Restored", "resolved"], removal_started: ["Removal started", "progress"],
        removal_undone: ["Removal undone", "progress"], removed: ["Removed", "closed"] })
    // A change-log row's reason/detail, from site_info place_log.
    function placeDetail(c) {
        var reason = (c.reason || "").replace(/[.\s]+$/, "")
        if (c.kind === "renamed") return "from “" + c.from + "” to “" + c.to + "”"
        if (c.kind === "removed") return (reason ? reason + " " : "") + "(started by " + who(c.by, c.by_name) + " on " + when(c.since, "d MMM") + ")"
        var pending = c.kind === "removal_started" && locations.some(function (l) { return l.code === c.code && l.state === "pending_removal" && l.removes_at === c.ends_at })
        return pending ? (reason ? reason + ". " : "") + "Hidden on " + when(c.ends_at, "d MMM") + " unless undone" : reason
    }
    // One change-log row in plain words: what the search matches and a screen reader says.
    function placeChange(c) {
        return [c.code, c.label, placeActions[c.kind][0], "by", who(c.by, c.by_name), when(c.ts), placeDetail(c)].join(" ")
    }
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
    // The conflicts count Integrity shows (History's strip too, once there are any): conflicts are the forks the core found.
    readonly property string conflicts: info.forks > 0 ? plural(info.forks, "conflict") : "no conflicts"
    // The newest anchor (site_info last_anchor, null = never). The app never reads LEZ: the tx is what
    // its recorder claimed, so the UI says "recorded", never "verified"; only the root is checked here.
    readonly property var lastAnchor: info.last_anchor || null
    readonly property int anchorStaleHours: 24
    // One state at a time, worst first: "mismatch", "never", "stale", or "" (recent and reproducible).
    readonly property string anchorState: !lastAnchor ? "never" : !lastAnchor.reproducible ? "mismatch"
        : now - lastAnchor.ts > anchorStaleHours * 3600 ? "stale" : ""
    // The anchor's own record is always newer than the heads it covers: it is not counted.
    function newerEvents(a) { return Math.max(0, (info.events || 0) - a.events_covered - 1) }
    function anchorSummary(a) {
        var newer = newerEvents(a)
        return "Last anchor recorded " + age(a.ts) + " (" + when(a.ts, now - a.ts < 6 * 86400 ? "ddd HH:mm" : "") + "). Covers "
            + plural(a.events_covered, "event") + "."
            + (newer ? " " + plural(newer, "newer event") + (newer === 1 ? " isn't" : " aren't") + " anchored yet." : "")
    }
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
    // How late the first acknowledge and the first fix claim were (the core's acked_ts / claimed_ts),
    // counted from the report like the live flags: History's red lines.
    function scars(i) {
        var out = []
        ;[["Acknowledged", i.acked_ts, info.sla_ack_h], ["Fixed", i.claimed_ts, info.sla_fix_h]].forEach(function (s) {
            var late = s[1] ? Math.ceil((s[1] - i.reported_ts) / 3600 - s[2]) : 0
            if (late > 0) out.push(s[0] + " " + late + " h past the " + s[2] + " h target.")
        })
        return out
    }
    function inTarget(i) { return i.claimed_ts - i.reported_ts <= info.sla_fix_h * 3600 }
    function span(sec) { var h = Math.round(sec / 3600); return h < 48 ? h + " h" : Math.floor(h / 24) + " d " + h % 24 + " h" }
    function closedVerb(i) { return i.status === "ConfirmedResolved" ? "Resolved" : "Closed" }
    // Wall-clock seconds, moved on each minute so relative times ("3 h ago") refresh on their own.
    property real now: Date.now() / 1000
    Timer { interval: 60000; running: true; repeat: true; onTriggered: root.now = Date.now() / 1000 }
    function hoursSince(ts) { return Math.floor((now - ts) / 3600) }
    function dur(h) { return h < 48 ? h + " h" : Math.floor(h / 24) + " d" }
    function age(ts) { var h = hoursSince(ts); return h < 1 ? "just now" : dur(h) + " ago" }
    // The card's red lines. SLA flags are computed by the core (site SLA hours); this only words them.
    // A finished issue keeps its scars instead.
    function flagLines(i) {
        if (isTerminal(i.status)) return scars(i)
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
    // mockup's words ("Comment needs a note first."). The core refuses these without a note.
    function unavailable(i, hasNote) {
        var note = hasNote ? "" : "note", steward = stewardWhy(i), why = []
        // a resident's steward actions are hidden, not explained: only staff get their stage reasons
        if (staff) ["Post update", "Claim fixed", "Won't fix"].forEach(function (a) { why.push([a, steward || note]) })
        // Reopen has its own reason box (Still broken? Reopen), so it never waits on the note
        why.push(["Confirm it's fixed", confirmWhy(i)], ["Reopen", reopenWhy(i)], ["Comment", note])
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
    // "How it was closed": the WhyCard of a finished issue, from the issue alone.
    function closedCard(i) {
        var p = i.progress || {}
        var verb = { ConfirmedResolved: "Confirmed fixed", ClosedWontfix: "Closed (won't fix)", Duplicate: "Marked duplicate" }[i.status]
        return {
            intro: verb + " by <b>" + (p.by === me.key ? "you" : esc(who(p.by, p.by_name))) + "</b>" + (p.by === i.reporter ? " (the reporter)" : "")
                 + " on " + when(closedAt(i)) + ", " + span(closedAt(i) - i.reported_ts) + " after the report.",
            steps: [], never: "", flags: scars(i),
            foot: i.reopen_count > 0 ? "Reopened " + i.reopen_count + "× before " + (i.status === "ConfirmedResolved" ? "the fix held." : "it was closed.") : ""
        }
    }
    function offBoardText(i) {
        return closedVerb(i) + " " + plural(Math.floor((Date.now() / 1000 - closedAt(i)) / 86400), "day") + " ago, so it is off the board, which shows the last "
             + windowDays(i.status) + " days. Nothing is deleted."
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
        var since = isTerminal(i.status) ? closedVerb(i) + " " + age(closedAt(i)) : "Reported " + age(i.reported_ts)
        return since + ". " + (i.former ? "The reporter is a former member. " : "") + waiting + nextLine(i)
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
            return o.id !== id && !isTerminal(o.status)
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

    // ---- icons: Lucide (lucide.dev, ISC licence, see THIRD_PARTY_LICENSES), 24x24 stroke set (2 stroke, round caps),
    // as SVG path data. `icons` is stroked, `iconFills` adds a filled part. `prog` is our own: a half-filled circle. ----
    function circ(cx, cy, r) { return "M" + (cx - r) + " " + cy + "a" + r + " " + r + " 0 1 0 " + 2 * r + " 0a" + r + " " + r + " 0 1 0 " + -2 * r + " 0" }
    function box(x, y, w, h, r) {
        const a = "a" + r + " " + r + " 0 0 1 "
        return "M" + (x + r) + " " + y + "h" + (w - 2 * r) + a + r + " " + r + "v" + (h - 2 * r) + a + -r + " " + r
             + "h" + (2 * r - w) + a + -r + " " + -r + "v" + (2 * r - h) + a + r + " " + -r + "z"
    }
    readonly property var icons: ({
        board: "M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2zM8 7v7M12 7v4M16 7v9",
        report: "M11 6a13 13 0 0 0 8.4-2.8A1 1 0 0 1 21 4v12a1 1 0 0 1-1.6.8A13 13 0 0 0 11 14H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2zM6 14a12 12 0 0 0 2.4 7.2 2 2 0 0 0 3.2-2.4A8 8 0 0 1 10 14M8 6v8",
        mega: "M11 6a13 13 0 0 0 8.4-2.8A1 1 0 0 1 21 4v12a1 1 0 0 1-1.6.8A13 13 0 0 0 11 14H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2zM6 14a12 12 0 0 0 2.4 7.2 2 2 0 0 0 3.2-2.4A8 8 0 0 1 10 14M8 6v8",
        members: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M16 3.128a4 4 0 0 1 0 7.744M22 21v-2a4 4 0 0 0-3-3.87M5 7a4 4 0 1 0 8 0a4 4 0 1 0 -8 0",
        anchor: "M12 6v16M19 13l2-1a9 9 0 0 1-18 0l2 1M9 11h6M10 4a2 2 0 1 0 4 0a2 2 0 1 0 -4 0",
        signal: "M4.9 16.1C1 12.2 1 5.8 4.9 1.9M7.8 4.7a6.14 6.14 0 0 0-.8 7.5M10 9a2 2 0 1 0 4 0a2 2 0 1 0 -4 0M16.2 4.8c2 2 2.26 5.11.8 7.47M19.1 1.9a9.96 9.96 0 0 1 0 14.1M9.5 18h5M8 22l4-11 4 11",
        user: "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2M8 7a4 4 0 1 0 8 0a4 4 0 1 0 -8 0",
        copy: "M10 8h10a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-10a2 2 0 0 1 2 -2zM4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
        dup: "M15 12L15 18M12 15L18 15M10 8h10a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-10a2 2 0 0 1 2 -2zM4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2",
        chev: "M9 18l6-6-6-6",
        chevl: "M15 18l-6-6 6-6",
        chevd: "M6 9l6 6 6-6",
        back: "M12 19l-7-7 7-7M19 12H5",
        x: "M18 6 6 18M6 6l12 12",
        plus: "M5 12h14M12 5v14",
        tick: "M20 6 9 17l-5-5",
        warn: "M21.73 18l-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3M12 9v4M12 17h.01",
        info: "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0M12 16v-4M12 8h.01",
        bell: "M10.268 21a2 2 0 0 0 3.464 0M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326",
        rep: "M11 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0",
        await: "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0M12 6v6l4 2",
        res: "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0M16 9l-5.5 5.5L8 12",
        clo: "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0M9 15L15 9",
        reject: "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0M15 9l-6 6M9 9l6 6",
        radio: "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0",
        radioOn: "M11 12a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0",
        cmt: "M22 17a2 2 0 0 1-2 2H6.828a2 2 0 0 0-1.414.586l-2.202 2.202A.71.71 0 0 1 2 21.286V5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2z",
        reopen: "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8M3 3v5h5",
        flag: "M4 22V4a1 1 0 0 1 .4-.8A6 6 0 0 1 8 2c3 0 5 2 7.333 2q2 0 3.067-.8A1 1 0 0 1 20 4v10a1 1 0 0 1-.4.8A6 6 0 0 1 16 16c-3 0-5-2-8-2a6 6 0 0 0-4 1.528",
        pin: "M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0M9 10a3 3 0 1 0 6 0a3 3 0 1 0 -6 0",
        eye: "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0M9 12a3 3 0 1 0 6 0a3 3 0 1 0 -6 0",
        update: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8M21 3v5h-5M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16M8 16H3v5",
        search: "M21 21l-4.34-4.34M3 11a8 8 0 1 0 16 0a8 8 0 1 0 -16 0",
        shield: "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z",
        hist: "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8M3 3v5h5M12 7v5l4 2",
        prog: "M2 12a10 10 0 1 0 20 0a10 10 0 1 0 -20 0"
    })
    readonly property var iconFills: ({ prog: "M12 6a6 6 0 0 1 0 12z" })
    // A named icon, `size` px square, drawn in `color`.
    component Icon: Item {
        id: icon
        property string name
        property real size: 16
        property color color: root.t.fg
        implicitWidth: size; implicitHeight: size
        Accessible.ignored: true
        Shape {
            width: 24; height: 24
            scale: icon.size / 24; transformOrigin: Item.TopLeft
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: icon.color; strokeWidth: 2; fillColor: "transparent"
                capStyle: ShapePath.RoundCap; joinStyle: ShapePath.RoundJoin
                PathSvg { path: root.icons[icon.name] || "" }
            }
            ShapePath { strokeColor: "transparent"; fillColor: icon.color; PathSvg { path: root.iconFills[icon.name] || "" } }
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
        property string iconName: ""       // optional leading icon, drawn in the label colour
        readonly property bool solid: kind === "primary" && enabled
        readonly property color ink: !enabled ? root.t.disabled : solid ? root.t.onPrimary : kind === "danger" ? root.t.danger : root.t.fg
        leftPadding: text === "" ? 9 : 14; rightPadding: leftPadding; topPadding: 6; bottomPadding: 6
        font.weight: Font.Medium
        contentItem: Label {     // the label wraps; the icon sits just before the centred text
            text: fb.text; font: fb.font; wrapMode: Text.Wrap
            horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
            leftPadding: fb.iconName === "" ? 0 : fb.text === "" ? 16 : 22
            color: fb.ink
            Icon {
                visible: fb.iconName !== ""; name: fb.iconName; size: 16; color: fb.ink
                x: (parent.width - parent.leftPadding - parent.contentWidth) / 2; anchors.verticalCenter: parent.verticalCenter
            }
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
    // The sidebar's tooltip: right of its parent, on the sidebar's tooltip tokens.
    component SideTip: ToolTip {
        id: tip
        delay: 300
        x: parent.width + 10; y: (parent.height - height) / 2
        contentItem: Label { text: tip.text; color: root.t.tipFg; font.pointSize: root.smallSize; font.weight: Font.Medium; wrapMode: Text.Wrap }
        background: Rectangle { radius: 6; color: root.t.tipBg; border.color: root.t.tipBd }
    }
    // One sidebar entry: a fill, bold text and an accent icon when current (no bar). `count` > 0
    // adds a pill (people waiting); `compact` is the icon-only rail, where the tooltip carries the label.
    component NavItem: AbstractButton {
        id: nav
        property string glyph
        property bool current: false
        property bool compact: false
        property int count: 0
        readonly property string waiting: count > 0 ? ", " + count + " waiting" : ""
        readonly property color ink: current ? root.t.sbActFg : root.t.sbFg
        readonly property color iconInk: current ? root.t.sbActIcon : root.t.sbIcon
        Layout.fillWidth: true
        implicitHeight: Math.max(44, contentItem.implicitHeight + topPadding + bottomPadding)
        leftPadding: compact ? 6 : 12; rightPadding: leftPadding; topPadding: 6; bottomPadding: 6
        hoverEnabled: true
        focusPolicy: Qt.StrongFocus
        Accessible.name: text + waiting
        HoverHandler { cursorShape: Qt.PointingHandCursor }
        SideTip { text: nav.text + nav.waiting; visible: nav.compact && (nav.hovered || nav.visualFocus) }
        background: Rectangle {
            radius: 8
            color: nav.current ? root.t.sbActBg : nav.hovered ? root.t.sbHover : "transparent"
            border.width: nav.visualFocus ? 2 : 0; border.color: root.t.focus
            Rectangle {   // the count pill: right of the label, or the rail icon's top-right corner
                visible: nav.count > 0
                width: Math.max(height, pill.implicitWidth + 10); height: nav.compact ? 18 : 20; radius: height / 2
                color: root.t.primary
                x: parent.width - width - (nav.compact ? 0 : 10)
                y: nav.compact ? 2 : (parent.height - height) / 2
                Label { id: pill; anchors.centerIn: parent; text: nav.count; color: root.t.onPrimary; font.pointSize: root.smallSize; font.weight: Font.DemiBold }
            }
        }
        contentItem: Item {
            implicitHeight: nav.compact ? 0 : lines.implicitHeight
            Icon { name: nav.glyph; size: 20; color: nav.iconInk; anchors.verticalCenter: parent.verticalCenter; x: nav.compact ? (parent.width - width) / 2 : 0 }
            Column {
                id: lines
                visible: !nav.compact
                x: 32; width: parent.width - 32 - (nav.count > 0 ? 36 : 0); anchors.verticalCenter: parent.verticalCenter
                Label { width: parent.width; text: nav.text; color: nav.ink; elide: Text.ElideRight; font.weight: nav.current ? Font.DemiBold : Font.Medium }
            }
        }
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
        readonly property real inset: icon !== "" ? 8 + 13 + 5 : 8
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

    // A red warning line: an SLA flag on a card, a scar in History.
    component FlagLine: Row {
        id: flag
        required property string modelData
        spacing: 6
        Icon { y: 2; name: "warn"; size: 15; color: root.t.danger }
        Label {
            width: flag.width - 21; wrapMode: Text.Wrap; text: flag.modelData
            font.pointSize: root.smallSize; font.weight: Font.DemiBold; color: root.t.danger
        }
    }

    // A 1 px dashed rule as wide as the item: between WhyCard steps, over a column footer, under a link.
    component DashLine: Item {   // an Item round the Shape: a path-sized Shape would feed its width back into a layout
        id: dash
        property color color: root.t.bd
        property var pattern: [3, 3]         // dash and gap, in px (the stroke is 1 px)
        implicitHeight: 1
        Shape {
            width: dash.width; height: 1
            ShapePath {
                strokeColor: dash.color; strokeWidth: 1; strokeStyle: ShapePath.DashLine; dashPattern: dash.pattern
                startX: 0; startY: 0.5
                PathLine { x: dash.width; y: 0.5 }
            }
        }
    }

    // Text that acts as a link: accent, semibold, 13 px, a focus ring of its own. `mono` is a code
    // (monospace, dotted underline). Inline in text, so it keeps the line's height, not 36 px.
    component LinkButton: AbstractButton {
        id: link
        property bool mono: false
        padding: 0
        hoverEnabled: true
        focusPolicy: Qt.StrongFocus
        HoverHandler { cursorShape: Qt.PointingHandCursor }
        Keys.onReturnPressed: clicked()
        Keys.onEnterPressed: clicked()
        contentItem: Label {
            text: link.text; textFormat: Text.PlainText; color: root.t.accent
            font.family: link.mono ? "monospace" : systemFont.font.family; font.pointSize: root.smallSize; font.weight: Font.DemiBold
        }
        background: Item {
            DashLine { visible: link.mono; y: parent.height + 1; width: parent.width; color: root.t.accent; pattern: link.hovered ? [1, 0] : [1, 2] }
            Rectangle { anchors { fill: parent; margins: -3 } radius: 4; color: "transparent"; border.width: link.visualFocus ? 2 : 0; border.color: root.t.focus }
        }
    }
    // A place's "Changes" link; the caller says what a click does.
    component ChangesLink: LinkButton {
        property string code
        text: "Changes"; Accessible.name: "Changes to " + code
    }
    // A place code that opens that place in History. A button of its own, so a click on it never
    // reaches the card it sits in.
    component PlaceLink: LinkButton {
        property string code
        text: code; mono: true
        Accessible.name: "History of " + code
        onClicked: root.showPlace(code)
    }

    // One issue card on the board.
    component IssueCard: ItemDelegate {
        id: card
        required property var issue
        property string placeText: root.place(issue)
        property string hintText: root.hint(issue)
        property var flags: root.flagLines(issue)                      // red lines
        property bool needsMe: root.needsMe(issue)                    // the viewer is the reporter of a claimed fix
        property bool current: !!root.selected && root.selected.issue.id === issue.id   // open in the pane: gets the stage strip
        property var stage: root.stageStyle(issue.status)
        signal opened()
        readonly property color ink: issue.location_retired ? root.t.muted : root.t.fg   // a retired place's card is quieter
        readonly property bool linked: issue.location !== "other"
        width: ListView.view.width
        topPadding: 10; bottomPadding: 12; rightPadding: 12
        leftPadding: current ? 16 : 12
        background: Frame {
            ring: card.visualFocus
            radius: 6
            border.width: card.visualFocus ? 2 : 1
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
            Item {   // the code is a PlaceLink laid over the first line, which is indented to make room
                width: parent.width; implicitHeight: placeLine.implicitHeight
                Label {
                    id: placeLine
                    width: parent.width; wrapMode: Text.Wrap; textFormat: Text.RichText; color: card.ink; font.weight: Font.DemiBold
                    text: card.linked ? "<div style='text-indent:" + (codeLink.width + 6) + "px'>" + root.esc(root.placeName(card.issue)) + "</div>"
                                      : root.mono(root.placeCode(card.issue)) + " " + root.esc(root.placeName(card.issue))
                }
                PlaceLink {
                    id: codeLink
                    visible: card.linked; code: card.issue.location
                    y: placeLine.baselineOffset - baselineOffset   // on the first line's baseline
                }
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
                delegate: FlagLine { width: card.availableWidth }
            }
        }
        onClicked: card.opened()
    }

    // The anchor at a glance (top of History): a dot and a line, then a link to Integrity. Green while
    // the anchor is recent and matches, amber when stale or never, red on a mismatch. Never "verified".
    component IntegrityStrip: Rectangle {
        id: strip
        readonly property var a: root.lastAnchor
        readonly property string state: root.anchorState
        readonly property color tone: state === "mismatch" ? root.t.sbOff : state === "" ? root.t.sbOk : root.t.sbConn
        readonly property string head: !a || state === "never" ? "Not anchored yet" : state === "mismatch" ? "Anchor mismatch"
            : state === "stale" ? "Anchor " + root.dur(root.hoursSince(a.ts)) + " old" : "Anchored " + root.age(a.ts)
        readonly property string detail: (a && state === "mismatch" ? "the record does not match the last anchor"
            : !a || state === "never" ? root.plural(root.info.events || 0, "event") + " recorded, none anchored"
            : root.plural(root.info.events || 0, "event") + (root.newerEvents(a) ? ", " + root.newerEvents(a) + " newer not anchored yet" : ""))
            + (root.info.forks > 0 ? " · " + root.conflicts : "")
        Layout.fillWidth: true; Layout.bottomMargin: 8
        implicitHeight: line.implicitHeight + 16
        radius: 8; color: root.t.well; border.color: root.t.bd
        Accessible.role: Accessible.StaticText; Accessible.name: head + ", " + detail
        Flow {   // the dot is a glyph, so the text and the link wrap as two units
            id: line
            x: 12; y: 8; width: parent.width - 24; spacing: 10
            Label {
                textFormat: Text.StyledText; font.pointSize: root.smallSize
                text: "<font color='" + strip.tone + "'>●</font>&nbsp; <b><font color='" + strip.tone + "'>" + strip.head + "</font></b> <font color='" + root.t.muted + "'>· " + strip.detail + "</font>"
            }
            LinkButton { text: "Open Integrity  ›"; Accessible.name: "Open Integrity"; onClicked: root.openPage("proof") }
        }
    }

    // A table's columns: each one's width (the `fill` ones also share what is left) and the `low` priority
    // ones that drop out when `compact`. History's drop out on a narrow page or beside the issue pane.
    readonly property var histTable: ({ w: [62, 170, 220, 150, 80, 170, 90, 60], fill: [1, 2, 5], low: [4, 5, 6, 7], compact: compactRows || paneOpen })
    readonly property var placeTable: ({ w: [56, 0, 110, 130, 84, 76, 296], fill: [1], low: [2, 4, 5], compact: compactRows })
    readonly property var logTable: ({ w: [64, 190, 140, 100, 200], fill: [1, 4], low: [3], compact: compactRows })
    // One table cell: a text, then whatever it holds (flag lines, chips). The header row uses it too.
    component HistCell: Column {
        required property int col
        property var cols: root.histTable
        property string text
        property int format: Text.PlainText
        property bool muted: false      // meta text: smaller, grey
        property color ink: muted ? root.t.muted : root.t.fg
        property bool bold: false
        property int align: Text.AlignLeft
        Layout.preferredWidth: cols.w[col]; Layout.fillWidth: cols.fill.indexOf(col) >= 0
        Layout.alignment: Qt.AlignTop
        visible: !(cols.compact && cols.low.indexOf(col) >= 0)
        spacing: 2
        Label {
            visible: parent.text !== ""
            width: parent.width; wrapMode: Text.Wrap; text: parent.text; textFormat: parent.format; horizontalAlignment: parent.align
            color: parent.ink; font.pointSize: parent.muted ? root.smallSize : root.baseSize; font.weight: parent.bold ? Font.DemiBold : Font.Normal
        }
    }
    component PlaceCell: HistCell { cols: root.placeTable }
    component LogCell: HistCell { cols: root.logTable }
    // A table's column titles: a tinted strip over its rows, inset like them.
    component TableHead: Rectangle {
        default property alias cells: heads.data
        Layout.fillWidth: true; implicitHeight: heads.implicitHeight + 16; color: root.t.well
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
        RowLayout { id: heads; x: 16; y: 8; width: parent.width - 32; spacing: 12 }
    }
    // One closed issue as a table row; a click opens the pane. A retired place's row is quieter.
    component HistRow: ItemDelegate {
        id: row
        required property var issue
        property string note            // "Confirmed: lid fitted (Asha)"
        property string hint            // "Resolved 4 Oct · ...", for screen readers
        readonly property var flags: root.scars(issue)
        readonly property var stage: root.stageStyle(issue.status)
        readonly property bool current: !!root.selected && root.selected.issue.id === issue.id
        readonly property color ink: issue.location_retired ? root.t.muted : root.t.fg
        width: ListView.view.width
        topPadding: 9; bottomPadding: 9; leftPadding: 16; rightPadding: 16
        background: Rectangle {
            color: row.current ? root.t.accentBg : row.hovered ? root.t.hover : row.issue.location_retired ? root.t.well : root.t.card
            border.width: row.visualFocus ? 2 : 0; border.color: root.t.focus
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
            Rectangle { visible: row.current; width: 5; height: parent.height; color: row.stage.edge }
        }
        Accessible.name: root.place(issue) + ": " + issue.text
        Accessible.description: issue.stage + ". " + hint + " " + flags.join(" ")
        contentItem: RowLayout {
            spacing: 12
            HistCell { col: 0; text: root.when(root.closedAt(row.issue), "d MMM"); ink: row.ink }
            HistCell {
                col: 1; ink: row.ink; format: Text.RichText
                text: root.mono(root.placeCode(row.issue)) + " " + root.esc(root.placeName(row.issue))
            }
            HistCell {
                col: 2; text: row.issue.text; ink: row.ink
                Repeater { model: row.flags; delegate: FlagLine { width: parent.width } }
            }
            HistCell {
                col: 3
                Flow {
                    width: parent.width; spacing: 4
                    StatusChip { label: row.issue.stage; stage: row.stage; icon: root.stageIcon(row.issue.status); maxWidth: parent.width }
                    StatusChip {
                        visible: row.issue.reopen_count > 0; maxWidth: parent.width
                        label: "Reopened " + row.issue.reopen_count + "×"; stage: root.t.closed; icon: "reopen"
                    }
                }
            }
            HistCell { col: 4; muted: true; text: row.issue.category }
            HistCell { col: 5; muted: true; text: row.note }
            HistCell { col: 6; muted: true; text: row.issue.reporter === root.me.key ? "you" : root.who(row.issue.reporter, row.issue.reporter_name) }
            HistCell { col: 7; muted: true; align: Text.AlignRight; text: root.span(root.closedAt(row.issue) - row.issue.reported_ts) }
        }
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
        RowLayout {   // the title wraps, the count chip stays beside it
            width: parent.width; spacing: 8
            Icon { Layout.alignment: Qt.AlignTop; Layout.topMargin: 2; name: head.icon; size: 16; color: head.tint }
            Label { Layout.fillWidth: true; text: head.title; font.weight: Font.DemiBold; wrapMode: Text.Wrap }
            Rectangle {
                Layout.alignment: Qt.AlignTop
                implicitWidth: Math.max(22, countText.implicitWidth + 14); implicitHeight: 20; radius: 10; color: root.t.closed.bg
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
                    text: root.esc(root.who(row.event.author, row.event.author_name)) + " " + root.mono(String(row.event.author).substr(0, 6)) + root.formerTag(row.event.former)
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
    // A finished issue shows "How it was closed" (root.closedCard()) in the same frame.
    component WhyCard: Rectangle {
        id: why
        required property var card      // whyCard(): {intro, steps: [{title, hint, meter?, done?}], never, flags?, foot?}
        required property var stage     // stageStyle(): header tint, icon and border
        property string title: "Why it isn't closed yet"
        property string icon: "info"
        implicitHeight: whyCol.implicitHeight + 2
        radius: 8; color: root.t.card; border.color: why.stage.bd
        Accessible.role: Accessible.Grouping; Accessible.name: why.title
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
                    Icon { name: why.icon; size: 16; color: why.stage.fg }
                    Label { Layout.fillWidth: true; wrapMode: Text.Wrap; text: why.title; font.weight: Font.DemiBold }
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
                        DashLine { visible: step.dashed; Layout.fillWidth: true }   // between the steps
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
                Repeater { model: why.card.flags || []; delegate: FlagLine { Layout.fillWidth: true; Layout.bottomMargin: 4 } }
                Help { visible: !!why.card.foot; text: why.card.foot || "" }
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
    // members, a long checkpoint). Content is a centred column of the page's width, capped (760 px
    // for forms, wider for Members) so forms don't sprawl across a wide window.
    component TabPage: ScrollView {
        id: page
        property int maxWidth: 760
        default property alias content: column.data
        contentWidth: availableWidth
        clip: true
        ColumnLayout { id: column; x: (page.availableWidth - width) / 2; width: Math.min(page.availableWidth, page.maxWidth); spacing: 8 }
    }
    // True when the content area (window minus sidebar) is under 880 px: rows of Members go two-line.
    readonly property bool compactRows: width - sidebarWidth < 880

    // A page's title (22 px) and its one-line lede.
    component PageHead: ColumnLayout {
        property string title
        property string lede
        Layout.fillWidth: true
        Layout.bottomMargin: 8
        spacing: 4
        Label { Layout.fillWidth: true; wrapMode: Text.Wrap; text: parent.title; font.pointSize: root.baseSize * 22 / 14; font.weight: Font.DemiBold }
        Label { visible: parent.lede !== ""; Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.StyledText; text: parent.lede; color: root.t.muted; font.pointSize: root.baseSize * 15 / 14 }
    }

    // A centred note for an empty list: an icon, a bold line, a muted explanation, then any buttons.
    component Blank: ColumnLayout {
        id: blank
        property string icon
        property string title
        property string text
        default property alias extra: blankActs.data
        Layout.fillWidth: true; Layout.topMargin: 28; Layout.bottomMargin: 28
        spacing: 4
        Icon { Layout.alignment: Qt.AlignHCenter; name: blank.icon; size: 24; color: root.t.muted }
        Label { Layout.alignment: Qt.AlignHCenter; text: blank.title; font.weight: Font.DemiBold; font.pointSize: root.baseSize * 16 / 14 }
        Label { Layout.fillWidth: true; horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap; color: root.t.muted; text: blank.text }
        Row { id: blankActs; Layout.alignment: Qt.AlignHCenter; Layout.topMargin: children.length > 0 ? 8 : 0; spacing: 8 }
    }

    // A row of joined toggle buttons, one pressed (History's Status and When). `current` is the
    // pressed index, bound by the caller; a press emits picked(index).
    component Segmented: Row {
        id: seg
        property var options: []
        property int current: 0
        property string name
        signal picked(int index)
        spacing: -1                          // neighbours share one border
        Accessible.role: Accessible.Grouping; Accessible.name: name
        Repeater {
            model: seg.options
            delegate: AbstractButton {
                id: opt
                required property string modelData
                required property int index
                readonly property bool on: seg.current === index
                readonly property real r: index === 0 ? 6 : 0
                readonly property real rr: index === seg.options.length - 1 ? 6 : 0
                text: modelData
                implicitHeight: 36; leftPadding: 12; rightPadding: 12
                hoverEnabled: true
                focusPolicy: Qt.StrongFocus
                Accessible.role: Accessible.Button; Accessible.checkable: true; Accessible.checked: on
                Keys.onReturnPressed: clicked()
                Keys.onEnterPressed: clicked()
                onClicked: seg.picked(index)
                contentItem: Label { text: opt.text; color: opt.on ? root.t.accent : root.t.fg; font.weight: opt.on ? Font.DemiBold : Font.Normal; verticalAlignment: Text.AlignVCenter }
                background: Rectangle {
                    z: opt.visualFocus ? 1 : 0
                    color: opt.on ? root.t.accentBg : opt.hovered ? root.t.hover : root.t.field
                    topLeftRadius: opt.r; bottomLeftRadius: opt.r; topRightRadius: opt.rr; bottomRightRadius: opt.rr
                    border.width: opt.visualFocus ? 2 : 1; border.color: opt.visualFocus ? root.t.focus : root.t.ctlBd
                }
            }
        }
    }

    // "Only my reports": one switch shared by the Board and History. A Binding, not a plain binding:
    // a click writes `checked`, and the other page's box must still follow.
    component MineCheck: FramedCheck {
        text: "Only my reports"; visible: root.approved
        Binding on checked { value: root.onlyMine }
        onToggled: root.onlyMine = checked
    }

    // A section title inside a page (18 px).
    component Heading: Label {
        Layout.topMargin: 12
        font.weight: Font.DemiBold
        font.pointSize: root.baseSize * 18 / 14
    }

    // A box the reader opens on demand: a 52 px header (chevron, title, a muted aside such as a count)
    // that is one button (Space or Enter toggles it), then a bordered body. `inner` is the flat version
    // inside another box. `open` lives only as long as the page, so it is remembered for the session
    // and nothing more.
    component CollapsibleSection: Rectangle {
        id: section
        property string title
        property string aside              // muted, after the title: "· 6" or "(site admin only)"
        property bool open: false
        property bool inner: false         // no box of its own: a rule on top and a 44 px header
        property int pad: 16               // margin round the body; 0 lets rows run edge to edge
        default property alias content: body.data
        signal toggled()
        Layout.fillWidth: true
        Layout.leftMargin: inner ? -16 : 0
        Layout.rightMargin: inner ? -16 : 0
        Layout.topMargin: inner ? 16 : 0
        implicitHeight: stack.implicitHeight + 2 * stack.x
        radius: 8; color: inner ? "transparent" : root.t.card
        border.color: inner ? "transparent" : root.t.bd
        ColumnLayout {
            id: stack
            x: section.inner ? 0 : 1; y: x; width: section.width - 2 * x
            spacing: 0
            Rectangle { visible: section.inner; Layout.fillWidth: true; implicitHeight: 1; color: root.t.bd }
            ItemDelegate {
                id: header
                Layout.fillWidth: true
                implicitHeight: section.inner ? 44 : 52
                leftPadding: 16; rightPadding: 16
                text: section.title
                Accessible.role: Accessible.Button
                Accessible.name: section.title + (section.aside ? " " + section.aside : "") + (section.open ? ", expanded" : ", collapsed")
                onClicked: { section.open = !section.open; section.toggled() }
                Keys.onReturnPressed: clicked()
                Keys.onEnterPressed: clicked()
                background: Rectangle {
                    readonly property real r: section.inner ? 0 : 7
                    color: header.hovered ? root.t.hover : section.inner ? "transparent" : root.t.well
                    topLeftRadius: r; topRightRadius: r
                    bottomLeftRadius: section.open ? 0 : r; bottomRightRadius: section.open ? 0 : r
                    border.width: header.visualFocus ? 2 : 0; border.color: root.t.focus
                    Rectangle { visible: section.open && !section.inner; anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
                }
                contentItem: RowLayout {
                    spacing: 10
                    Icon { name: "chev"; size: 16; color: root.t.muted; rotation: section.open ? 90 : 0 }
                    Label { text: header.text; elide: Text.ElideRight; font.weight: Font.DemiBold; font.pointSize: section.inner ? root.baseSize : root.baseSize * 16 / 14 }
                    Label { visible: section.aside !== ""; Layout.fillWidth: true; text: section.aside; elide: Text.ElideRight; color: root.t.muted; font.pointSize: root.smallSize }
                    Item { visible: section.aside === ""; Layout.fillWidth: true }
                }
            }
            ColumnLayout {
                id: body
                visible: section.open
                Layout.fillWidth: true
                Layout.margins: section.pad
                spacing: section.pad > 0 ? 12 : 0
            }
        }
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
        Help { text: rule.hint }
    }

    // A visible label above whatever is put inside (every field has one, not only a placeholder):
    // 14 px semibold, or 13 px when `compact`; `sub` is a muted aside, "Note (what was seen, done or why)".
    component FormRow: ColumnLayout {
        id: form
        property string label
        property string sub
        property bool compact: false
        Layout.fillWidth: true
        spacing: 4
        Label {
            visible: form.label !== ""; Layout.fillWidth: true; wrapMode: Text.Wrap
            textFormat: Text.RichText; font.weight: Font.DemiBold; font.pointSize: form.compact ? root.smallSize : root.baseSize
            text: root.esc(form.label) + (form.sub ? " <span style='font-weight:400;font-size:" + root.smallSize + "pt;color:" + root.t.muted + "'>" + root.esc(form.sub) + "</span>" : "")
        }
    }

    // A tinted notice (info blue, `err` red, or a `tone`: "warn" amber, "plain" neutral) with an icon, wrapped text, and optionally one button
    // on the right: a text one (`buttonText`) or an icon-only one (`buttonIcon`).
    component Flash: Rectangle {
        id: flash
        property string text
        property string lead               // a bold red first sentence
        property bool err: false
        property string tone: err ? "err" : "info"
        readonly property var ink: ({ info: [root.t.accentBg, root.t.accentBd, root.t.accent], err: [root.t.dangerBg, root.t.dangerBd, root.t.danger],
                                      warn: [root.t.warnBg, root.t.warnBd, root.t.warn], plain: [root.t.well, root.t.bd, root.t.muted] })[tone]
        property bool bold: false          // the whole text bold (red in an error)
        property string icon: err ? "warn" : "info"
        property string buttonText
        property string buttonIcon
        property string buttonName: buttonText     // what a screen reader says for the button
        property string buttonEffect               // set: the button asks first (ConfirmButton)
        signal activated()
        Layout.fillWidth: true
        implicitHeight: row.implicitHeight + 22
        radius: 6
        color: ink[0]
        border.color: ink[1]
        RowLayout {
            id: row
            anchors { fill: parent; leftMargin: 13; rightMargin: 13; topMargin: 11; bottomMargin: 11 }
            spacing: 10
            Icon { Layout.alignment: Qt.AlignTop; Layout.topMargin: 2; name: flash.icon; size: 16; color: flash.ink[2] }
            Label {
                Layout.fillWidth: true; Layout.alignment: Qt.AlignTop; wrapMode: Text.Wrap; textFormat: Text.RichText
                font.bold: flash.bold; color: flash.err && flash.bold ? root.t.danger : root.t.fg
                text: (flash.lead ? "<b><font color='" + flash.ink[2] + "'>" + root.esc(flash.lead) + "</font></b> " : "") + root.esc(flash.text)
            }
            ConfirmButton {
                visible: flash.buttonText !== "" || flash.buttonIcon !== ""
                Layout.alignment: Qt.AlignTop
                text: flash.buttonText; iconName: flash.buttonIcon
                name: flash.buttonName; effect: flash.buttonEffect
                onConfirmed: flash.activated()
            }
        }
    }

    // A wrapped paragraph. Plain text on purpose: "<YOUR_PUBLIC_ACCOUNT>" must not be read as a tag.
    component Note: Label {
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        textFormat: Text.PlainText
    }

    // A muted 13 px paragraph under a field or title.
    component Help: Note { color: root.t.muted; font.pointSize: root.smallSize }

    // A fingerprint in a monospace chip; `big` for the one read aloud at the kiosk.
    component Fingerprint: Rectangle {
        id: fp
        property string text
        property bool big: false
        implicitWidth: fpText.implicitWidth + (big ? 22 : 14); implicitHeight: big ? 32 : 24
        radius: 6; color: root.t.well; border.color: root.t.bd
        Label {
            id: fpText
            anchors.centerIn: parent; text: fp.text; font.family: "monospace"; textFormat: Text.PlainText
            font.pointSize: fp.big ? root.baseSize * 18 / 14 : root.smallSize; font.weight: fp.big ? Font.DemiBold : Font.Normal
            font.letterSpacing: fp.big ? 1 : 0
        }
    }

    // Home's "how it works" strip: four numbered cards, two per row when the content is under 700 px.
    component HowItWorks: GridLayout {
        columns: root.width - root.sidebarWidth < 700 ? 2 : 4
        columnSpacing: 10; rowSpacing: 10
        Layout.fillWidth: true
        Layout.bottomMargin: 12
        Accessible.name: "How it works"
        Repeater {
            model: [
                { n: "1", title: "Report", text: "Say what is wrong and where." },
                { n: "2", title: "Seen", text: "A steward acknowledges it." },
                { n: "3", title: "Fixed?", text: "A steward says what was done." },
                { n: "4", title: "You confirm", text: "Only then is it resolved." }
            ]
            delegate: Rectangle {
                id: step
                required property var modelData
                Layout.fillWidth: true; Layout.fillHeight: true; Layout.preferredWidth: 1
                implicitHeight: stepCol.implicitHeight + 26
                radius: 8; color: root.t.card; border.color: root.t.bd
                ColumnLayout {
                    id: stepCol
                    anchors { fill: parent; margins: 13 }
                    spacing: 2
                    Rectangle {
                        Layout.bottomMargin: 4
                        implicitWidth: 26; implicitHeight: 26; radius: 13; color: root.t.accentBg; border.color: root.t.accentBd
                        Label { anchors.centerIn: parent; text: step.modelData.n; color: root.t.accent; font.pointSize: root.smallSize; font.bold: true }
                    }
                    Label { Layout.fillWidth: true; text: step.modelData.title; wrapMode: Text.Wrap; font.pointSize: root.baseSize * 15 / 14; font.weight: Font.DemiBold }
                    Label { Layout.fillWidth: true; text: step.modelData.text; wrapMode: Text.Wrap; color: root.t.muted; font.pointSize: root.smallSize }
                    Item { Layout.fillHeight: true }
                }
            }
        }
    }

    // One numbered step of the Anchor page: a round number, a title, then the step's own content.
    component Step: RowLayout {
        id: step
        property int n
        property string title
        default property alias content: stepBody.data
        Layout.fillWidth: true
        Layout.topMargin: 8
        spacing: 12
        Rectangle {
            Layout.alignment: Qt.AlignTop
            implicitWidth: 30; implicitHeight: 30; radius: 15; color: root.t.accentBg; border.color: root.t.accentBd
            Label { anchors.centerIn: parent; text: step.n; color: root.t.accent; font.bold: true }
        }
        ColumnLayout {
            id: stepBody
            Layout.fillWidth: true; Layout.alignment: Qt.AlignTop
            spacing: 6
            Label { Layout.topMargin: 2; text: step.title; font.weight: Font.DemiBold; font.pointSize: root.baseSize * 16 / 14 }
        }
    }

    // "+N more" under a capped list; the caller keeps `expanded` and flips it on click.
    component MoreToggle: ItemDelegate {
        id: more
        property bool expanded: false
        implicitHeight: Math.max(44, implicitContentHeight + topPadding + bottomPadding)
        leftPadding: 12; rightPadding: 12
        Accessible.role: Accessible.Button
        Accessible.name: text + (expanded ? ", expanded" : ", collapsed")
        Keys.onReturnPressed: clicked()
        Keys.onEnterPressed: clicked()
        background: Rectangle {
            radius: 6; color: more.hovered ? root.t.hover : root.t.card
            border.width: more.visualFocus ? 2 : 1; border.color: more.visualFocus ? root.t.focus : root.t.ctlBd
        }
        contentItem: RowLayout {
            spacing: 8
            Icon { name: "chev"; size: 16; color: root.t.accent; rotation: more.expanded ? -90 : 90 }
            Label { Layout.fillWidth: true; text: more.text; wrapMode: Text.Wrap; color: root.t.accent; font.weight: Font.DemiBold }
        }
    }

    // A row of the change log's table, from one site_info place_log entry.
    component LogRow: Rectangle {
        id: log
        required property var modelData
        readonly property var action: root.placeActions[modelData.kind]
        Layout.fillWidth: true; implicitHeight: cells.implicitHeight + 20; color: "transparent"
        Accessible.role: Accessible.StaticText; Accessible.name: root.placeChange(modelData)
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
        RowLayout {
            id: cells
            x: 16; y: 10; width: parent.width - 32; spacing: 12
            LogCell { col: 0; muted: true; text: root.when(log.modelData.ts, "d MMM") }
            LogCell { col: 1; format: Text.RichText; text: root.mono(log.modelData.code) + " " + root.esc(log.modelData.label || "") }
            LogCell { col: 2; StatusChip { label: log.action[0]; stage: root.t[log.action[1]] } }
            LogCell { col: 3; muted: true; text: root.who(log.modelData.by, log.modelData.by_name) }
            LogCell { col: 4; text: root.placeDetail(log.modelData) }
        }
    }

    // The one multi-line text box: visible frame, scrolls, optional placeholder. Read-only boxes
    // are selectable and get a Copy button (more buttons can be put inside, beside it). With
    // `enterAccepts`, Return emits accepted() and Shift+Return starts a new line.
    component FramedTextArea: ColumnLayout {
        id: fta
        property alias text: area.text
        property alias placeholder: area.placeholderText
        property alias wrap: area.wrapMode
        property bool readOnly: false
        property bool mono: false
        property bool enterAccepts: false
        property int maxLength: 0         // characters; 0 = no limit here (the core checks its own)
        property string name              // what a screen reader says for the box
        property int boxHeight: 70
        default property alias extra: actions.data
        signal accepted()
        function enter(e) { if (fta.enterAccepts && !(e.modifiers & Qt.ShiftModifier)) fta.accepted(); else e.accepted = false }
        Layout.fillWidth: true
        spacing: 4
        ScrollView {
            Layout.fillWidth: true
            Layout.preferredHeight: fta.boxHeight
            clip: true
            background: Frame { ring: area.activeFocus; off: !fta.enabled }
            TextArea {
                id: area
                readOnly: fta.readOnly
                selectByMouse: true
                wrapMode: TextEdit.WrapAnywhere
                color: root.ink(fta.enabled)
                placeholderTextColor: root.t.ph
                font.family: fta.mono ? "monospace" : systemFont.font.family
                padding: 8
                background: null
                Accessible.name: fta.name
                onTextChanged: if (fta.maxLength > 0 && length > fta.maxLength) remove(fta.maxLength, length)
                Keys.onReturnPressed: function (e) { fta.enter(e) }
                Keys.onEnterPressed: function (e) { fta.enter(e) }
            }
        }
        RowLayout {
            id: actions
            visible: fta.readOnly && area.text !== ""
            spacing: 8
            FramedButton {
                text: "Copy"; iconName: "copy"
                Accessible.name: "Copy " + fta.name
                onClicked: root.copyText(area.text)
            }
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

    // A person the admin can give a role (waiting for approval, or removed and `again`): who they are
    // (the content), then, for the admin only, Grant resident, or a name and Grant steward.
    component GrantRow: ColumnLayout {
        id: grant
        required property var modelData
        required property int index
        property bool again: false
        default property alias who: whoCell.data
        readonly property string suffix: again ? " again" : ""
        readonly property string person: modelData.name || modelData.fingerprint
        Layout.fillWidth: true
        spacing: 0
        Rectangle { visible: grant.index > 0; Layout.fillWidth: true; implicitHeight: 1; color: root.t.bd }
        GridLayout {
            Layout.fillWidth: true; Layout.margins: 12; Layout.leftMargin: 16; Layout.rightMargin: 16
            columns: root.compactRows ? 2 : 3; columnSpacing: 12; rowSpacing: 8
            ColumnLayout { id: whoCell; Layout.fillWidth: true; spacing: 4 }
            ActionButton {
                visible: root.isAdmin
                text: "Grant resident" + grant.suffix; Accessible.name: text + ": " + grant.person
                onClicked: root.run("grant_role", [grant.modelData.key, "resident", staffName.text])
            }
            LabelledField {
                id: staffName
                visible: root.isAdmin
                Layout.fillWidth: root.compactRows; Layout.columnSpan: root.compactRows ? 2 : 1
                Layout.preferredWidth: 330
                name: "Name, optional for a resident"
                placeholder: "Name, e.g. Ravi Kumar (a steward needs one)"
                buttonText: "Grant steward" + grant.suffix
                buttonEnabled: staffName.text.trim().length > 0
                onSubmitted: root.run("grant_role", [grant.modelData.key, "steward", staffName.text])
            }
        }
    }

    // An ActionButton that asks first: with an `effect` (one short line), a click swaps it for that
    // line over the confirm button and Cancel. With no `effect` it fires at once (Flash, LabelledField).
    component ConfirmButton: ColumnLayout {
        id: cb
        property alias text: first.text
        property alias kind: first.kind
        property alias iconName: first.iconName
        property alias allowed: first.allowed
        property string name: text           // what a screen reader says for the first button
        property string confirmText: "Confirm"
        property string effect
        property bool armed: false
        signal confirmed()
        function submit() { first.submit() }
        spacing: 6
        Help { visible: cb.armed; text: cb.effect }
        RowLayout {
            Layout.fillHeight: true
            spacing: 8
            ActionButton {
                id: first; visible: !cb.armed; Layout.fillHeight: true; Accessible.name: cb.name
                onClicked: { if (!cb.effect) return cb.confirmed(); cb.armed = true; yes.forceActiveFocus() }   // keyboard focus follows to Confirm
            }
            ActionButton { id: yes; visible: cb.armed; Layout.fillHeight: true; allowed: cb.allowed; text: cb.confirmText; kind: "danger"; onClicked: { cb.armed = false; cb.confirmed() } }
            FramedButton { visible: cb.armed; Layout.fillHeight: true; text: "Cancel"; onClicked: cb.armed = false }
        }
    }

    // A row's inline confirm step: the question, any `extra` choices, the reason the core requires
    // (every member can read it), what happens, then the confirm button and Cancel.
    component ReasonPanel: ColumnLayout {
        id: rp
        property string title
        property string effect
        property string okText
        property string kind: "danger"
        property string reasonName           // what a screen reader says for the reason field
        property bool withReason: true
        property bool ready: true            // the `extra` choices are complete
        property alias reason: reasonField.text
        default property alias extra: extras.data
        signal confirmed()
        signal cancelled()
        // The owner calls this when the step opens or changes: a fresh reason, and the cursor in it.
        function reset() { reasonField.text = ""; Qt.callLater(function () { if (rp.visible && rp.withReason) reasonField.forceActiveFocus() }) }
        Layout.fillWidth: true; Layout.maximumWidth: 560; Layout.rightMargin: 16; Layout.bottomMargin: 12
        spacing: 6
        Label { Layout.fillWidth: true; wrapMode: Text.Wrap; font.weight: Font.DemiBold; textFormat: Text.PlainText; text: rp.title }
        ColumnLayout { id: extras; visible: children.length > 0; Layout.fillWidth: true; spacing: 6 }
        FormRow {
            visible: rp.withReason
            label: "Reason (required)"; compact: true
            FramedField {
                id: reasonField
                Layout.fillWidth: true; maximumLength: 500
                placeholderText: "Everyone on the site can read this."
                Accessible.name: rp.reasonName
                onAccepted: ok.submit()
            }
        }
        Help { text: rp.effect }
        RowLayout {
            spacing: 8
            ActionButton {
                id: ok
                text: rp.okText; kind: rp.kind
                allowed: rp.ready && (!rp.withReason || reasonField.text.trim() !== "")
                onClicked: rp.confirmed()
            }
            FramedButton { text: "Cancel"; onClicked: rp.cancelled() }
        }
    }

    // A text field with its submit button glued on: the "paste id / type name, then act" row.
    // `label` sits above it. `large` is the 40 px first-run size, `mono` a monospace field.
    component LabelledField: FormRow {
        id: lf
        property alias text: field.text
        property alias placeholder: field.placeholderText
        property alias echoMode: field.echoMode
        property alias fieldEnabled: field.enabled
        property string name: label
        property bool large: false
        property bool mono: false
        property string buttonText: "Go"
        property string buttonKind: "default"
        property string buttonEffect                // set: the button asks first (ConfirmButton)
        property string missing                     // what still blocks the button, in words ("" = ready)
        property bool buttonEnabled: missing === ""
        signal submitted()
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            FramedField {
                id: field
                Layout.fillWidth: true
                Layout.preferredHeight: lf.large ? 40 : -1
                maximumLength: 500
                font.family: lf.mono ? "monospace" : systemFont.font.family
                Accessible.name: lf.name
                onAccepted: go.submit()
            }
            ConfirmButton {
                id: go
                Layout.preferredHeight: lf.large ? 40 : -1
                text: lf.buttonText; kind: lf.buttonKind; allowed: lf.buttonEnabled; effect: lf.buttonEffect
                onConfirmed: lf.submitted()
            }
        }
        Help { visible: lf.missing !== ""; text: lf.missing }
    }

    // Group title inside a list (a place group, a History month). The list's `pad` is the row inset.
    // With an `aside` (History's counts) the title is darker and full size, the aside muted after it.
    component SectionHeader: Rectangle {
        required property string section
        property string aside
        readonly property int pad: ListView.view.pad
        width: ListView.view.width
        height: title.implicitHeight + (aside ? 18 : 12)
        color: root.t.well
        Label {
            id: title
            x: parent.pad; width: parent.width - 2 * parent.pad; anchors.verticalCenter: parent.verticalCenter
            wrapMode: Text.Wrap; textFormat: Text.StyledText
            text: "<b>" + root.esc(parent.section) + "</b>" + (parent.aside ? "&nbsp;&nbsp; <font color='" + root.t.muted + "'>" + root.esc(parent.aside) + "</font>" : "")
            font.pointSize: parent.aside ? root.baseSize : root.smallSize; color: parent.aside ? root.t.fg : root.t.muted
        }
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
    }

    // One selectable row of the Report picker: a radio mark, the code (monospace), the name. A retired
    // place is muted and italic and never becomes the pick; with `explains` it stays clickable so the
    // caller can say why (see PlaceBrowser.retiredPlace).
    component PlaceChoice: ItemDelegate {
        id: choice
        property string code
        property string note              // upright, after the name: "(retired)"
        property bool picked: false
        property bool retired: false
        property bool explains: false
        implicitHeight: Math.max(40, implicitContentHeight + topPadding + bottomPadding)   // a long (Hindi) name wraps
        topPadding: 5; bottomPadding: 5; leftPadding: 12; rightPadding: 12
        enabled: !retired || explains
        Accessible.role: Accessible.RadioButton
        Accessible.checked: choice.picked
        background: Rectangle {
            color: choice.picked ? root.t.accentBg : choice.hovered ? root.t.hover : root.t.page
            Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
        }
        contentItem: RowLayout {
            spacing: 8
            Icon { name: choice.picked ? "radioOn" : "radio"; size: 16; color: choice.picked ? root.t.accent : root.t.ctlBd }
            Label {
                visible: choice.code !== ""; text: choice.code; textFormat: Text.PlainText
                font.family: "monospace"; font.weight: Font.DemiBold; font.pointSize: root.smallSize; font.italic: choice.retired
                color: choice.retired ? root.t.muted : root.t.fg
            }
            Label {
                Layout.fillWidth: true; text: choice.text; wrapMode: Text.Wrap
                font.weight: choice.picked ? Font.DemiBold : Font.Normal; font.italic: choice.retired
                color: choice.retired ? root.t.muted : root.t.fg
            }
            Label { visible: choice.note !== ""; text: choice.note; font.pointSize: root.smallSize; color: root.t.muted }
        }
    }

    // Filters (search, group, retired) + a count + a framed list of the matching places, grouped
    // under headers. The caller draws the rows (`rowDelegate`, may be given `listHeader` above them).
    // `bare` drops the frame and indents the filters: for a list that fills a box edge to edge.
    component PlaceBrowser: ColumnLayout {
        id: pb
        property int maxHeight: 260
        property bool bare: false
        property Component rowDelegate
        property Component listHeader
        // Report picker only: the retired place the user just tried to pick (null = none). While
        // set, a notice under the list explains it and offers to report it as "Other".
        property var retiredPlace: null
        signal reportOther()
        readonly property var places: root.placesView(search.text, groupFilter.currentIndex > 0 ? groupFilter.currentText : "", showRetired.checked)
        readonly property int total: root.placesView("", "", showRetired.checked).length
        readonly property int pad: bare ? 16 : 0
        Layout.fillWidth: true
        spacing: 6
        RowLayout {
            Layout.fillWidth: true
            Layout.margins: pb.pad; Layout.topMargin: pb.bare ? 12 : 0; Layout.bottomMargin: pb.bare ? 6 : 0
            spacing: 12
            FormRow {
                label: "Search"; compact: true
                FramedField { id: search; Layout.fillWidth: true; placeholderText: "code, name or group"; Accessible.name: "Search places" }
            }
            FormRow {
                label: "Group"; compact: true
                Layout.fillWidth: false
                FramedCombo {
                    id: groupFilter
                    Layout.preferredWidth: 200
                    Accessible.name: "Filter by group"
                    model: ["All groups"].concat(root.groupNames)
                }
            }
            FramedCheck { id: showRetired; objectName: "showRetired"; text: "Show retired places"; Layout.alignment: Qt.AlignBottom }
        }
        Rectangle { visible: pb.bare; Layout.fillWidth: true; implicitHeight: 1; color: root.t.bd }
        Label {
            Layout.leftMargin: pb.pad; Layout.topMargin: pb.bare ? 2 : 0; Layout.bottomMargin: pb.bare ? 2 : 0
            font.pointSize: root.smallSize
            color: root.t.muted
            text: pb.places.length === 0 ? "No places match. Clear the search or pick another group."
                                         : pb.places.length + " of " + pb.total + " places"
        }
        Frame {
            Layout.fillWidth: true
            Layout.topMargin: pb.bare ? 0 : 2
            Layout.preferredHeight: Math.min(list.contentHeight - 1, pb.maxHeight) + (pb.bare ? 0 : 2)
            radius: pb.bare ? 0 : 6
            border.width: pb.bare ? 0 : 1
            border.color: pb.bare ? root.t.bd : root.t.ctlBd
            ListView {
                id: list
                property int pad: pb.bare ? 16 : 12       // row inset, read by SectionHeader
                anchors.fill: parent
                anchors.margins: pb.bare ? 0 : 1
                clip: true
                cacheBuffer: 100000      // keep every row alive: text typed into a row survives scrolling
                model: pb.places
                delegate: pb.rowDelegate
                section.property: pb.bare ? "" : "group"   // a table has a Group column instead
                section.delegate: SectionHeader {}
                header: Loader { width: list.width; sourceComponent: pb.listHeader }
                ScrollBar.vertical: ScrollBar {}
            }
        }
        Flash {
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

    // ---------- sidebar: replaces the old header and tab bar ----------
    Rectangle {
        id: sidebar
        z: 5
        width: root.sidebarOpen ? 216 : 80; height: parent.height
        color: root.t.sb
        readonly property bool open: root.sidebarOpen
        // the toggle sits on the site-name line: card top + border and padding + half the first line
        readonly property real nameY: siteCard.parent.y + siteCard.y + (open ? 11 : 9) + siteTitle.contentHeight / siteTitle.lineCount / 2
        readonly property string deliveryText: root.deliveryLabel(root.me.delivery)
        readonly property color on: root.t.sbFg
        readonly property string fp: root.me.fingerprint || ""
        Rectangle { anchors.right: parent.right; width: 1; height: parent.height; color: root.t.sbEdge }
        Rectangle {   // the overlay's shadow
            visible: root.overlay && root.overlayOpen
            x: parent.width; width: 14; height: parent.height
            gradient: Gradient { orientation: Gradient.Horizontal; GradientStop { position: 0; color: "#40000000" } GradientStop { position: 1; color: "transparent" } }
        }
        ColumnLayout {
            anchors { fill: parent; leftMargin: sidebar.open ? 12 : 6; rightMargin: sidebar.open ? 21 : 7; topMargin: sidebar.open ? 16 : 12; bottomMargin: sidebar.open ? 16 : 12 }
            spacing: sidebar.open ? 16 : 12
            RowLayout {   // brand
                Layout.alignment: sidebar.open ? Qt.AlignLeft : Qt.AlignHCenter
                Layout.leftMargin: sidebar.open ? 4 : 0
                spacing: 10
                Item {
                    implicitWidth: 28; implicitHeight: 28
                    Accessible.ignored: true
                    // the module icon (colour bell, transparent), shipped next to Main.qml
                    Image { anchors.fill: parent; source: "src/icons/pukaar.png"; sourceSize: Qt.size(56, 56); fillMode: Image.PreserveAspectFit }
                }
                Label { visible: sidebar.open; text: "Pukaar"; color: sidebar.on; font.pointSize: root.baseSize * 18 / 14; font.bold: true }
            }
            Rectangle {   // site card
                id: siteCard
                Layout.fillWidth: sidebar.open
                Layout.preferredWidth: sidebar.open ? -1 : 40
                Layout.alignment: Qt.AlignHCenter
                implicitHeight: cardCol.implicitHeight + (sidebar.open ? 22 : 18)
                radius: 8; color: root.t.sbCard; border.color: root.t.sbLine
                ColumnLayout {
                    id: cardCol
                    anchors { fill: parent; leftMargin: sidebar.open ? 12 : 1; rightMargin: sidebar.open ? 12 : 1; topMargin: sidebar.open ? 11 : 9; bottomMargin: sidebar.open ? 11 : 9 }
                    spacing: 2
                    Label {
                        id: siteTitle
                        Layout.fillWidth: true
                        readonly property string full: !root.inSite ? "not in a site yet" : root.info.name || "Waiting for site data…"
                        text: sidebar.open ? full : root.initials(root.inSite ? root.info.name : "") || "…"   // the 40 px card cannot hold a name: initials, the tooltip has the rest
                        color: sidebar.on; font.weight: Font.DemiBold
                        wrapMode: Text.WrapAnywhere
                        horizontalAlignment: sidebar.open ? Text.AlignLeft : Text.AlignHCenter
                        HoverHandler { id: cardHover }
                        SideTip { parent: siteCard; text: siteTitle.full; visible: !sidebar.open && cardHover.hovered }
                    }
                }
            }
            ColumnLayout {   // sections
                visible: root.inSite
                Layout.fillWidth: true
                spacing: 4
                Repeater {
                    model: [
                        { label: "Board", icon: "board", page: "board" },
                        { label: "Report", icon: "report", page: "report" },
                        { label: "History", icon: "hist", page: "history" },
                        { label: "People", icon: "members", page: "people", count: root.isAdmin ? root.pending.length : 0 },   // the admin's waiting count
                        { label: "Places", icon: "pin", page: "places" }
                    ].filter(function (e) { return root.approved || e.page !== "report" })   // a pending key cannot report
                    delegate: NavItem {
                        required property var modelData
                        text: modelData.label; glyph: modelData.icon
                        current: root.page === modelData.page; compact: !sidebar.open
                        count: modelData.count || 0
                        onClicked: root.openPage(modelData.page)
                    }
                }
            }
            Item { Layout.fillHeight: true }
            Rectangle { Layout.fillWidth: true; height: 1; color: root.t.sbLine }   // foot
            ColumnLayout {
                Layout.fillWidth: true; Layout.topMargin: -4
                spacing: 12
                GridLayout {   // delivery: a dot and the status, the reason in red when it failed
                    Layout.fillWidth: true
                    columns: sidebar.open ? 3 : 1; columnSpacing: 8; rowSpacing: 4
                    Rectangle {
                        Layout.alignment: Qt.AlignHCenter | Qt.AlignTop; Layout.topMargin: sidebar.open ? 3 : 0
                        implicitWidth: 10; implicitHeight: 10; radius: 5
                        color: root.failed(root.me.delivery) ? root.t.sbOff : sidebar.deliveryText === "online" ? root.t.sbOk : root.t.sbConn
                    }
                    Label { Layout.alignment: sidebar.open ? Qt.AlignTop : Qt.AlignHCenter; text: "delivery:"; color: root.t.sbMuted; font.pointSize: root.smallSize }
                    Label {
                        Layout.fillWidth: true; Layout.alignment: Qt.AlignTop
                        text: sidebar.deliveryText; wrapMode: Text.Wrap
                        horizontalAlignment: sidebar.open ? Text.AlignLeft : Text.AlignHCenter
                        color: root.failed(root.me.delivery) ? root.t.sbDanger : sidebar.on
                        font.pointSize: root.smallSize; font.weight: Font.Medium
                    }
                }
                AbstractButton {   // you: name, fingerprint, role; opens Profile
                    id: you
                    visible: root.inSite
                    Layout.fillWidth: true; Layout.leftMargin: -6; Layout.rightMargin: -6   // the padding holds the hover fill and focus ring
                    padding: 6
                    hoverEnabled: true
                    focusPolicy: Qt.StrongFocus
                    Accessible.name: "You: " + (root.me.name || "pseudonym") + ", " + root.roleName + ". Open Profile"
                    HoverHandler { cursorShape: Qt.PointingHandCursor }
                    SideTip { text: "Profile"; visible: !sidebar.open && (you.hovered || you.visualFocus) }
                    onClicked: root.openPage("profile")
                    background: Rectangle {
                        radius: 8
                        color: root.page === "profile" ? root.t.sbActBg : you.hovered ? root.t.sbHover : "transparent"
                        border.width: you.visualFocus ? 2 : 0; border.color: root.t.focus
                    }
                    contentItem: GridLayout {
                        columns: sidebar.open ? 3 : 1; columnSpacing: 10; rowSpacing: 4
                        Rectangle {
                            Layout.alignment: Qt.AlignHCenter | Qt.AlignTop
                            implicitWidth: 32; implicitHeight: 32; radius: 16; color: root.t.sbAv
                            Label {
                                anchors.centerIn: parent; visible: !!root.me.name
                                text: root.initials(root.me.name)
                                color: root.t.sbFg; font.pointSize: root.smallSize; font.bold: true
                            }
                            Icon { anchors.centerIn: parent; visible: !root.me.name; name: "user"; size: 18; color: root.t.sbFg }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true; Layout.preferredWidth: 0; Layout.minimumWidth: 0   // the rail sets the width: a long fingerprint or role must wrap, not widen the column
                            Layout.maximumWidth: Infinity   // not the children's: they cap at this column's width, which would pin it at 0 in the rail
                            spacing: 2
                            readonly property int align: sidebar.open ? Qt.AlignLeft : Qt.AlignHCenter
                            Label { visible: sidebar.open; text: "you:"; color: root.t.sbMuted; font.pointSize: root.smallSize }
                            Label {
                                visible: sidebar.open; Layout.fillWidth: true
                                text: root.me.name || "pseudonym"; color: root.t.sbFg; wrapMode: Text.WrapAnywhere
                                font.pointSize: root.smallSize; font.weight: Font.DemiBold
                            }
                            Label {
                                Layout.alignment: parent.align; Layout.maximumWidth: parent.width; wrapMode: Text.WrapAnywhere
                                text: sidebar.fp; color: root.t.sbFg; font.pointSize: root.smallSize; font.family: "monospace"
                            }
                            Rectangle {   // role chip; a key with no role yet is "pending", or "removed" after a revoke
                                readonly property bool pending: !root.me.role
                                Layout.alignment: parent.align; Layout.maximumWidth: parent.width
                                implicitWidth: roleText.implicitWidth + (sidebar.open ? 16 : 12); implicitHeight: roleText.implicitHeight
                                radius: 12; color: "transparent"; border.color: pending ? root.t.sbDanger : root.t.sbCtl
                                Label {
                                    id: roleText
                                    width: parent.width; horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap
                                    lineHeight: 20; lineHeightMode: Text.FixedHeight
                                    text: sidebar.open && root.unapproved ? "pending: read your fingerprint at the kiosk" : root.roleName
                                    color: parent.pending ? root.t.sbDanger : root.t.sbFg; font.pointSize: root.smallSize
                                }
                            }
                        }
                        Icon { visible: sidebar.open; Layout.alignment: Qt.AlignVCenter; name: "chev"; size: 14; color: root.t.sbIcon }
                    }
                }
            }
        }
    }
    Rectangle {   // the scrim behind the overlaid sidebar: a click closes it
        visible: root.overlay && root.overlayOpen
        z: 4; x: 216; width: parent.width - 216; height: parent.height
        color: "#4d000000"
        MouseArea { anchors.fill: parent; onClicked: root.overlayOpen = false }
    }
    Shortcut { sequence: "Escape"; enabled: root.overlayOpen; onActivated: root.overlayOpen = false }
    Button {   // the collapse toggle: a 28 px knob in a 36 px hit area, on the divider and the site-name line
        id: toggle
        z: 20
        x: sidebar.width - 20; y: sidebar.nameY - 18
        width: 36; height: 36; padding: 4
        hoverEnabled: true
        Accessible.name: sidebar.open ? "Collapse sidebar" : "Expand sidebar"
        onClicked: root.toggleSidebar()
        SideTip { text: toggle.Accessible.name; visible: toggle.hovered || toggle.visualFocus }
        background: Rectangle { radius: 18; color: "transparent"; border.width: toggle.visualFocus ? 2 : 0; border.color: root.t.focus }
        contentItem: Rectangle {
            radius: 14; color: toggle.hovered ? root.t.sbHover : root.t.page; border.color: root.t.sbCtl
            Icon { anchors.centerIn: parent; name: sidebar.open ? "chevl" : "chev"; size: 16; color: root.t.sbFg }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        anchors.leftMargin: root.sidebarWidth + 16
        spacing: 8

        Flash {   // the last refused action; stays until dismissed or the next action replaces it
            visible: root.message !== ""
            Accessible.role: Accessible.AlertMessage
            err: true; bold: true
            text: root.message.replace(/^error: /, "")   // the red warning icon already says so
            buttonIcon: "x"; buttonName: "Dismiss this error"
            onActivated: root.message = ""
        }
        Flash {   // after an action: where it went (a reopened issue is back on the board)
            visible: root.notice !== ""
            icon: "reopen"
            text: root.notice
            buttonIcon: "x"; buttonName: "Dismiss this notice"
            onActivated: root.notice = ""
        }
        Flash {   // forks: someone signed two versions of their log (the threat model says: flag it)
            visible: root.inSite && root.info.forks > 0
            err: true; bold: true; icon: "shield"
            text: "Integrity warning: " + root.info.forks + " conflicting history pair(s) found. Someone signed two versions of their log, so devices may show different boards."
        }
        Flash {   // joined, but the genesis hasn't arrived: wrong id, or nobody online yet
            visible: root.inSite && !root.info.name
            text: "Waiting for site data… If this lasts, check the site id, or wait for a member to come online."
            buttonText: "Leave this site"
            onActivated: root.leave()
        }
        Flash {   // joined and waiting for a role: what to do, and what comes after
            visible: root.awaitingApproval
            text: "Waiting for the admin to approve you. Your fingerprint: " + (root.me.fingerprint || "") + ". Read it aloud at the kiosk. After approval you can report faults, confirm fixes and comment."
        }
        Flash {   // our role was revoked: by whom, when, why, and what is left
            visible: root.removed
            err: true
            text: root.removed ? "Your role was removed by " + root.who(root.me.revoked.by, root.me.revoked.by_name) + " on " + root.when(root.me.revoked.ts, "d MMM")
                                 + ": " + root.me.revoked.reason + " You can still read the board, and your past reports stay. To take part again, ask the admin to grant you a role." : ""
        }
        Flash {
            visible: root.inSite && root.syncing
            err: true; icon: "update"
            text: "Restoring your earlier reports from the network. Actions are paused until they arrive, so your new ones can't conflict with them."
            buttonText: "Skip waiting (history lost)"
            buttonEffect: "Your earlier reports may then be lost for good."
            onActivated: root.run("skip_history_sync", [])
        }

        // ---------- first run: join, create or restore ----------
        TabPage {
            visible: !root.inSite
            Layout.fillWidth: true
            Layout.fillHeight: true
            PageHead { title: "Report a problem, and see it fixed."; lede: "Pukaar keeps every report until the person who raised it says it works." }
            HowItWorks {}
            CollapsibleSection {
                title: "Join a site"
                open: true
                LabelledField {
                    id: joinField
                    label: "Site id"; sub: "(shown on the kiosk)"
                    placeholder: "64 hex characters, e.g. 45f1dbbf…a050"
                    large: true; mono: true
                    buttonText: "Join"; buttonKind: "primary"
                    onSubmitted: root.run("site_join", [joinField.text])
                }
                Help { text: "Ask a steward, or read it from the kiosk screen." }
            }
            CollapsibleSection {
                title: "Create a site"; aside: "(site admin only)"
                Help { text: "Nothing is published until you press Create site. Places can be added later too, on the Places page." }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 16
                    FormRow {
                        label: "Site name"
                        FramedField { id: siteName; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Site name"; placeholderText: "e.g. Dhun Camp" }
                    }
                    FormRow {   // staff are always named: this goes into the genesis as the admin's name
                        label: "Your name, as residents will see it"
                        FramedField { id: adminName; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Your name"; placeholderText: "e.g. John" }
                    }
                }
                FormRow {
                    label: "Categories"; sub: "(what can go wrong)"
                    Flow {
                        Layout.fillWidth: true
                        spacing: 8
                        Repeater {
                            model: setup.categories
                            delegate: Rectangle {   // a pill with its own remove button
                                id: cat
                                required property string modelData
                                required property int index
                                implicitWidth: catRow.implicitWidth + 12; implicitHeight: 36
                                radius: 18; color: root.t.card; border.color: root.t.ctlBd
                                RowLayout {
                                    id: catRow
                                    x: 12; height: parent.height; spacing: 0
                                    Label { text: cat.modelData; textFormat: Text.PlainText }
                                    Button {
                                        implicitWidth: 36; implicitHeight: 34
                                        Accessible.name: "Remove category " + cat.modelData
                                        onClicked: setup.categories = setup.categories.filter(function (c, i) { return i !== cat.index })
                                        background: Rectangle {
                                            anchors.centerIn: parent; width: 30; height: 30; radius: 15
                                            color: parent.hovered ? root.t.hover : "transparent"
                                            border.width: parent.visualFocus ? 2 : 0; border.color: root.t.focus
                                        }
                                        contentItem: Icon { name: "x"; size: 14; color: root.t.muted }
                                    }
                                }
                            }
                        }
                    }
                }
                LabelledField {
                    id: newCategory
                    label: "Add a category"; compact: true
                    placeholder: "e.g. electricity"
                    buttonText: "Add"
                    buttonEnabled: setup.canAddCategory
                    onSubmitted: { setup.categories = setup.categories.concat([newCategory.text.trim()]); newCategory.text = "" }
                }
                FormRow {
                    label: "Places (" + setup.places.count + ")"
                    Help { text: "Each place has a group (for the picker), a short code people can say aloud, and a name. Codes can't change later." }
                    Frame {   // the table: column titles, then the draft rows or an empty note
                        Layout.fillWidth: true
                        implicitHeight: table.implicitHeight + 2
                        clip: true
                        ColumnLayout {
                            id: table
                            x: 1; y: 1; width: parent.width - 2
                            spacing: 0
                            Rectangle {
                                Layout.fillWidth: true; implicitHeight: 34; color: root.t.well
                                Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
                                RowLayout {   // the placeholders vanish once a row is filled, so the titles stay
                                    anchors { fill: parent; leftMargin: 18; rightMargin: 8 }
                                    spacing: 6
                                    Label { Layout.preferredWidth: 170; text: "Group"; font.pointSize: root.smallSize; font.weight: Font.DemiBold; color: root.t.muted }
                                    Label { Layout.preferredWidth: 90; text: "Code"; font.pointSize: root.smallSize; font.weight: Font.DemiBold; color: root.t.muted }
                                    Label { Layout.fillWidth: true; text: "Name"; font.pointSize: root.smallSize; font.weight: Font.DemiBold; color: root.t.muted }
                                    Item { implicitWidth: 36 }
                                }
                            }
                            ColumnLayout {
                                visible: setup.places.count === 0
                                Layout.fillWidth: true; Layout.margins: 12; Layout.topMargin: 18; Layout.bottomMargin: 18
                                spacing: 0
                                Label { Layout.alignment: Qt.AlignHCenter; text: "No places yet"; font.weight: Font.DemiBold }
                                Label { Layout.fillWidth: true; horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap; color: root.t.muted; text: "Add them one by one, or start from the sample and edit it." }
                            }
                            Repeater {
                                model: setup.places
                                delegate: RowLayout {
                                    id: draftRow
                                    required property int index
                                    required property string group
                                    required property string code
                                    required property string label
                                    Layout.fillWidth: true; Layout.margins: 8; Layout.bottomMargin: 0
                                    spacing: 6
                                    // Writes straight into the ListModel, so other rows keep their typed text.
                                    FramedField { Layout.preferredWidth: 170; maximumLength: 500; text: draftRow.group; placeholderText: "e.g. Water points"; Accessible.name: "Place " + (draftRow.index + 1) + " group"; onTextEdited: setup.edit(draftRow.index, "group", text) }
                                    FramedField { Layout.preferredWidth: 90; maximumLength: 500; text: draftRow.code; placeholderText: "e.g. W-01"; Accessible.name: "Place " + (draftRow.index + 1) + " code"; onTextEdited: setup.edit(draftRow.index, "code", text) }
                                    FramedField { Layout.fillWidth: true; maximumLength: 500; text: draftRow.label; placeholderText: "e.g. Tap, dining hall"; Accessible.name: "Place " + (draftRow.index + 1) + " name"; onTextEdited: setup.edit(draftRow.index, "label", text) }
                                    FramedButton { iconName: "x"; Accessible.name: "Remove place " + (draftRow.index + 1); onClicked: setup.places.remove(draftRow.index) }
                                }
                            }
                            Item { visible: setup.places.count > 0; implicitHeight: 8 }
                        }
                    }
                    Flow {
                        Layout.fillWidth: true
                        spacing: 8
                        FramedButton { iconName: "plus"; text: "Add a place"; onClicked: setup.places.append({ group: "", code: "", label: "" }) }
                        FramedButton { text: "Start from the Dhun sample"; onClicked: setup.fillSample() }
                        FramedButton { enabled: setup.places.count > 0; text: "Clear places"; onClicked: setup.places.clear() }
                    }
                }
                CollapsibleSection {
                    inner: true
                    title: "Service rules (optional)"
                    RuleField { id: ruleAck; label: "Acknowledge within (hours)"; text: "12"; hint: "After this, an unacknowledged report is flagged red." }
                    RuleField { id: ruleFix; label: "Fix within (hours, after acknowledging)"; text: "48"; hint: "After this, the issue shows as overdue." }
                    RuleField { id: ruleMax; label: "Open reports per person (max)"; text: "10"; hint: "Stops one person flooding the board; more can be filed once some are resolved." }
                }
                CollapsibleSection {
                    id: advanced
                    inner: true
                    title: "Advanced: edit the settings as JSON"
                    onToggled: if (open) advancedJson.text = setup.genesisJson()
                    Help { text: "Starts from the form above and replaces it while this section is open. Close it to go back to the form." }
                    FramedTextArea { id: advancedJson; name: "Site settings, JSON"; mono: true; boxHeight: 160 }
                }
                RowLayout {
                    Layout.fillWidth: true; Layout.topMargin: 8
                    spacing: 12
                    ActionButton {
                        text: "Create site"; kind: "primary"
                        Layout.preferredHeight: 40
                        allowed: advanced.open ? adminName.text.trim() !== "" : setup.problem === ""
                        onClicked: root.run("site_create", [advanced.open ? setup.withAdmin(advancedJson.text) : setup.genesisJson()], null,
                                            "Start with fewer places, then add the rest after creating, via Places → Add a place.")
                    }
                    Help { visible: !advanced.open && setup.problem !== ""; text: setup.problem }
                }
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
                Help { text: "Paste the backup made on the Profile page, on your other device or before you reinstalled." }
                FormRow {
                    label: "Identity backup"
                    FramedTextArea {
                        id: importBlob
                        name: "Identity backup"
                        mono: true; boxHeight: 76
                        placeholder: "e.g. pukaar-id-1:…"
                    }
                }
                LabelledField {
                    id: importPassword
                    label: "Backup password"
                    placeholder: "the password you set when exporting"
                    echoMode: TextInput.Password
                    buttonText: "Import identity"; buttonKind: "primary"
                    buttonEffect: "It replaces any key on this device."
                    missing: importBlob.text.trim() === "" ? "Paste the backup first." : importPassword.text === "" ? "Type its password." : ""
                    onSubmitted: {
                        root.run("import_identity", [importBlob.text, importPassword.text],
                                 function () { importBlob.text = ""; importPassword.text = "" })
                    }
                }
            }
        }

        // ---------- in a site ----------
        RowLayout {   // the page, and the issue pane beside it (Board and History)
            visible: root.inSite
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 8
            StackLayout {
                visible: !(root.narrow && root.paneOpen)   // narrow: the pane takes the page's place
                currentIndex: root.pages.indexOf(root.page)
                Layout.fillWidth: true
                Layout.fillHeight: true

                // Board (the brief's three stages); finished issues by the view rule above
                ColumnLayout {
                    id: boardPage
                    // what the check counts: open, finished within the window, and older (History)
                    readonly property var tally: {
                        var c = [0, 0, 0]
                        root.issues.forEach(function (i) { if (root.mineOk(i)) c[!root.isTerminal(i.status) ? 0 : root.onBoard(i) ? 1 : 2]++ })
                        return c
                    }
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 16
                        MineCheck {}
                        Label {
                            Layout.fillWidth: true; horizontalAlignment: Text.AlignRight; wrapMode: Text.Wrap
                            text: boardPage.tally[0] + " open · " + boardPage.tally[1] + " recently done · " + boardPage.tally[2] + " older"
                            font.pointSize: root.smallSize; color: root.t.muted
                        }
                    }
                    Flash {   // we are an admin on notice: when and how our role changes, and why
                        readonly property var n: root.me.notice
                        visible: !!n && !n.sealed && n.deadline > root.now
                        Layout.bottomMargin: 8
                        tone: "warn"; icon: "warn"
                        lead: n ? (n.outcome === "remove" ? "You are removed from the site on " : "You become a steward on ") + root.when(n.deadline) + "." : ""
                        text: n ? "Reason: “" + n.reason + "” You keep admin powers until then." : ""
                    }
                    Flash {   // the record no longer matches its anchor: every member sees it, on every Board visit
                        visible: root.anchorState === "mismatch"
                        Layout.bottomMargin: 8
                        err: true
                        lead: "The site record does not match its last anchor."
                        text: "Someone may have changed history after it was anchored."
                        buttonText: "Open Integrity"
                        onActivated: root.openPage("proof")
                    }
                    Flash {   // an empty board says why, and what to do
                        visible: root.boardCount === 0
                        Layout.bottomMargin: 8
                        text: !root.approved ? "No reports yet."
                            : root.onlyMine ? "You haven't reported anything yet. Use Report to raise a fault."
                                            : "No reports yet. Use Report to raise the first one."
                        buttonText: root.approved ? "Report a fault" : ""
                        onActivated: root.openPage("report")
                    }
                    RowLayout {   // the four columns
                        spacing: 12
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Repeater {
                            model: [
                                { title: "Reported", status: "Open", statuses: ["Open", "Acknowledged"], subtitle: "Waiting for a steward to see it", empty: "No reports yet." },
                                { title: "In progress", status: "InProgress", statuses: ["InProgress", "AwaitingConfirmation"], subtitle: "A steward is on it", empty: "Nothing in progress." },
                                { title: "Resolved", status: "ConfirmedResolved", statuses: ["ConfirmedResolved"], subtitle: "Confirmed fixed · last 14 days",
                                  empty: "Nothing resolved yet.", quiet: "Nothing fixed in the last 14 days.", older: "fixed issues" },
                                { title: "Closed without fix", status: "ClosedWontfix", statuses: ["ClosedWontfix", "Duplicate"], subtitle: "Won't fix or duplicate · last 30 days",
                                  empty: "Nothing closed.", quiet: "Nothing closed in the last 30 days.", older: "closed without fix" }
                            ]
                            delegate: Rectangle {
                                id: col
                                required property var modelData
                                readonly property bool finished: root.isTerminal(modelData.status)
                                readonly property var cards: root.inColumn(modelData.statuses, true)
                                readonly property int older: finished ? root.inColumn(modelData.statuses, false).length : 0
                                readonly property int over: finished ? Math.max(0, cards.length - root.boardCap) : 0
                                property bool showAll: false      // "+N more" opened; forgotten once the page is left
                                onVisibleChanged: if (!visible) showAll = false
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                Layout.preferredWidth: 1   // equal shares of the width...
                                Layout.minimumWidth: 160   // ...but never thinner than this
                                color: root.t.well; radius: 8; border.color: root.t.bd
                                ColumnLayout {
                                    anchors { fill: parent; margins: 8 }
                                    spacing: 0
                                    ColumnHeader {   // the count is everything in the window, so it matches "+N more"
                                        Layout.fillWidth: true; Layout.topMargin: 4; Layout.leftMargin: 4
                                        title: col.modelData.title; subtitle: col.modelData.subtitle; count: col.cards.length
                                        icon: root.stageIcon(col.modelData.status); tint: root.stageStyle(col.modelData.status).fg
                                    }
                                    Label {   // an empty column says so, in a dashed box
                                        id: empty
                                        visible: col.cards.length === 0
                                        Layout.fillWidth: true; padding: 12; horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap
                                        text: root.onlyMine ? "None of your reports here yet." : col.older > 0 ? col.modelData.quiet : col.modelData.empty
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
                                        id: cardList
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        clip: true
                                        spacing: 8
                                        model: col.over > 0 && !col.showAll ? col.cards.slice(0, root.boardCap) : col.cards
                                        delegate: IssueCard {
                                            required property var modelData
                                            issue: modelData
                                            onOpened: root.openIssue(modelData.id)
                                        }
                                        footer: Column {   // "+N more" under the last card, then the way to the older ones
                                            width: cardList.width
                                            topPadding: col.cards.length > 0 && (col.over > 0 || col.older > 0) ? 8 : 0
                                            spacing: 4
                                            MoreToggle {
                                                visible: col.over > 0
                                                width: parent.width
                                                expanded: col.showAll
                                                text: col.showAll ? "Show fewer" : "+" + col.over + " more from the last " + root.windowDays(col.modelData.status) + " days"
                                                onClicked: col.showAll = !col.showAll
                                            }
                                            ItemDelegate {
                                                id: olderLink
                                                visible: col.older > 0
                                                width: parent.width; implicitHeight: Math.max(40, implicitContentHeight + topPadding + bottomPadding)
                                                topPadding: 8; bottomPadding: 8; leftPadding: 6; rightPadding: 6
                                                Accessible.role: Accessible.Link
                                                Accessible.name: col.older + " older " + col.modelData.older + ", nothing deleted. Open History"
                                                onClicked: root.showPlace("")
                                                Keys.onReturnPressed: clicked()
                                                Keys.onEnterPressed: clicked()
                                                background: Rectangle {
                                                    color: olderLink.hovered ? root.t.hover : "transparent"; bottomLeftRadius: 6; bottomRightRadius: 6
                                                    border.width: olderLink.visualFocus ? 2 : 0; border.color: root.t.focus
                                                    DashLine { width: parent.width; color: root.t.ctlBd; pattern: [4, 3] }
                                                }
                                                contentItem: Flow {
                                                    spacing: 6
                                                    Label {
                                                        width: Math.min(implicitWidth, parent.width); wrapMode: Text.Wrap; textFormat: Text.StyledText
                                                        text: "<b><font color='" + root.t.fg + "'>" + col.older + " older</font></b> " + col.modelData.older + ", nothing deleted."
                                                        font.pointSize: root.smallSize; color: root.t.muted
                                                    }
                                                    Row {
                                                        spacing: 2
                                                        Label { text: "History"; color: root.t.accent; font.pointSize: root.smallSize; font.weight: Font.DemiBold }
                                                        Icon { anchors.verticalCenter: parent.verticalCenter; name: "chev"; size: 14; color: root.t.accent }
                                                    }
                                                }
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
                    PageHead { title: "What's wrong?"; lede: "A line or two is enough. A steward will see it on the board." }
                    FormRow {
                        label: "Category"
                        FramedCombo { id: category; Accessible.name: "What is wrong (category)"; model: root.categories; Layout.fillWidth: true; Layout.maximumWidth: 372 }
                    }
                    // maximumLength counts characters; the core's 500 limit is bytes (Hindi is 3 B/char),
                    // so a long non-Latin line comes back as "error: too long" instead. Return sends,
                    // Shift+Return starts a new line.
                    FormRow {
                        label: "In a line or two"
                        FramedTextArea {
                            id: reportText
                            boxHeight: 76; maxLength: 500; enterAccepts: true
                            wrap: TextEdit.WrapAtWordBoundaryOrAnywhere
                            placeholder: "e.g. Tap near tent 4 gives no water"
                            name: "What is wrong"
                            onAccepted: reportBtn.submit()
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
                                code: modelData.code
                                text: modelData.label
                                note: modelData.state !== "active" ? "(" + root.placeState(modelData) + ")" : ""
                                retired: modelData.state !== "active"
                                explains: true
                                picked: reportPage.pick === modelData.code
                                onClicked: modelData.state !== "active" ? reportPage.retired = modelData : reportPage.choose(modelData.code)
                            }
                        }
                    }
                    Help {
                        text: reportPage.pick === "" ? "No place picked yet."
                            : reportPage.pick === "other" ? "Other place: describe it below."
                            : "Picked: " + root.placeLabel(reportPage.pick)
                    }
                    FormRow {
                        visible: reportPage.pick !== ""
                        label: reportPage.pick === "other" ? "Describe the place (required)" : "Landmark"
                        sub: reportPage.pick === "other" ? "" : "(optional)"
                        FramedField {
                            id: landmark; Layout.fillWidth: true; maximumLength: 500
                            Accessible.name: reportPage.pick === "other" ? "Describe the place" : "Landmark"
                            placeholderText: reportPage.pick === "other" ? "e.g. pipe behind tent 4, by the neem tree" : "e.g. the left tap, behind the water tank"
                        }
                    }
                    RowLayout {
                        Layout.fillWidth: true; Layout.topMargin: 4
                        spacing: 12
                        ActionButton {
                            id: reportBtn
                            text: "Report"; kind: "primary"
                            Layout.preferredHeight: 40
                            allowed: reportPage.missing === ""
                            onClicked: root.run("report", [category.currentText, reportPage.pick, landmark.text, reportText.text],
                                                function () { reportText.text = ""; landmark.text = ""; reportPage.pick = ""; root.openPage("board") })
                        }
                        Help { visible: reportPage.missing !== ""; text: reportPage.missing }
                    }
                    RowLayout {   // no name yet: one line to Profile
                        visible: !root.me.name
                        Layout.topMargin: 8
                        spacing: 6
                        Help { Layout.fillWidth: false; text: "Reporting as " + root.who(root.me.key, root.me.name) + " ·" }
                        LinkButton { text: "Set a name"; onClicked: root.openPage("profile") }
                    }
                }

                // People (everyone in the site): who has which role, and who was removed, by whom and why.
                // The admin also grants pending keys after the fingerprint is read aloud, and revokes.
                // Only the admin's "Waiting for approval" (or everyone else's member list) is open at first.
                TabPage {
                    maxWidth: 1040
                    enabled: !root.syncing
                    PageHead { title: "People"; lede: root.isAdmin ? "Approve people and manage their roles." : "Who is in this site." }
                    CollapsibleSection {
                        visible: root.isAdmin
                        title: "Waiting for approval"; aside: "· " + root.pending.length
                        open: true; pad: 0
                        Flash { Layout.margins: 16; Layout.bottomMargin: 12; icon: "mega"; text: "Grant a role only after the person reads this fingerprint aloud." }
                        Blank {   // empty: say so and say when someone appears
                            visible: root.pending.length === 0
                            Layout.topMargin: 12
                            icon: "res"; title: "No one is waiting."; text: "New people appear here after they join with the site id."
                        }
                        Repeater {
                            model: root.pending
                            delegate: GrantRow {
                                id: pendingRow
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 10
                                    Fingerprint { big: true; text: pendingRow.modelData.fingerprint }
                                    Label {
                                        Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.PlainText
                                        text: pendingRow.modelData.name || "(no name)"
                                        color: pendingRow.modelData.name ? root.t.fg : root.t.muted
                                        font.weight: pendingRow.modelData.name ? Font.DemiBold : Font.Normal
                                    }
                                }
                            }
                        }
                    }
                    CollapsibleSection {
                        objectName: "memberList"
                        title: "Members"; aside: "· " + root.members.length
                        open: !root.isAdmin; pad: 0
                        Help {
                            visible: root.isSuper && root.superCount >= 2
                            Layout.margins: 16; Layout.bottomMargin: 6
                            text: "Two super admins already, the most allowed."
                        }
                        Repeater {
                            model: root.members
                            // Who changes what: an admin, the residents and stewards; a super admin, also the admins.
                            delegate: ColumnLayout {
                                id: memberRow
                                objectName: "memberRow-" + modelData.fingerprint
                                required property var modelData
                                required property int index
                                readonly property bool own: modelData.key === root.me.key
                                readonly property bool admin: modelData.role === "Admin"
                                readonly property bool sup: !!modelData.super_admin
                                readonly property bool elevated: admin || modelData.role === "Steward"
                                readonly property string person: root.who(modelData.key, modelData.name)
                                readonly property var running: admin ? root.noticeOf(modelData) : null   // their notice, still running
                                readonly property bool canChange: root.isSuper && admin && !sup && !running
                                property string panel                // "" or the open step: "revoke", "change" or "transfer"
                                onPanelChanged: { revokeStep.reset(); changeStep.reset() }
                                Layout.fillWidth: true
                                spacing: 0
                                Rectangle { visible: memberRow.index > 0; Layout.fillWidth: true; implicitHeight: 1; color: root.t.bd }
                                GridLayout {
                                    Layout.fillWidth: true; Layout.margins: 10; Layout.leftMargin: 16; Layout.rightMargin: 16
                                    columns: root.compactRows ? 3 : 4; columnSpacing: 12; rowSpacing: 8
                                    Fingerprint { Layout.alignment: Qt.AlignTop; Layout.preferredWidth: 92; text: memberRow.modelData.fingerprint }
                                    Item {   // the role chip has its own column; a narrow row puts it after the name
                                        visible: !root.compactRows
                                        Layout.alignment: Qt.AlignTop
                                        Layout.preferredWidth: 100; implicitHeight: roleChip.height
                                        StatusChip { id: roleChip; label: memberRow.modelData.role; stage: memberRow.elevated ? root.t.progress : root.t.closed }
                                    }
                                    ColumnLayout {
                                        Layout.fillWidth: true; Layout.alignment: Qt.AlignTop
                                        spacing: 0
                                        Flow {
                                            id: nameFlow
                                            Layout.fillWidth: true; spacing: 6
                                            Label {
                                                width: Math.min(implicitWidth, nameFlow.width)
                                                wrapMode: Text.Wrap; textFormat: Text.PlainText; font.weight: Font.DemiBold
                                                text: memberRow.person
                                            }
                                            StatusChip { visible: memberRow.sup; label: "Super admin"; stage: root.t.progress; strong: true }
                                            StatusChip { visible: root.compactRows; label: memberRow.modelData.role; stage: memberRow.elevated ? root.t.progress : root.t.closed }
                                        }
                                        Label { visible: memberRow.own; text: "you"; color: root.t.muted; font.pointSize: root.smallSize }
                                        Help { visible: text !== ""; Layout.topMargin: 4; text: root.noticeLine(memberRow.modelData) }
                                        FlagLine { id: rejected; visible: rejected.modelData !== ""; Layout.fillWidth: true; Layout.topMargin: 4; modelData: root.rejectedLine(memberRow.modelData) }
                                    }
                                    RowLayout {   // only the controls this viewer may use; nothing at all for residents and stewards
                                        visible: root.isAdmin
                                        Layout.alignment: Qt.AlignTop | Qt.AlignRight; Layout.fillHeight: false   // buttons their own height, not the row's
                                        spacing: 8
                                        Help {
                                            visible: memberRow.admin && !root.isSuper
                                            Layout.fillWidth: false; Layout.maximumWidth: 240
                                            text: "Only a super admin can change admins."
                                        }
                                        ConfirmButton {
                                            visible: root.isSuper && !memberRow.admin && !!memberRow.modelData.name
                                            text: "Grant admin"; name: text + ": " + memberRow.person; confirmText: "Confirm grant"
                                            effect: "They become an admin now."
                                            onConfirmed: root.run("grant_role", [memberRow.modelData.key, "admin", memberRow.modelData.name])
                                        }
                                        ActionButton {
                                            visible: memberRow.canChange
                                            text: "Change admin"; Accessible.name: text + ": " + memberRow.person
                                            onClicked: memberRow.panel = "change"
                                        }
                                        ConfirmButton {
                                            visible: memberRow.canChange && root.superCount < 2 && !!memberRow.modelData.name
                                            text: "Add super admin"; name: text + ": " + memberRow.person; confirmText: "Confirm"
                                            effect: "They can change admins too, from now on."
                                            onConfirmed: root.run("add_super_admin", [memberRow.modelData.key])
                                        }
                                        ConfirmButton {
                                            visible: root.isSuper && !!memberRow.running
                                            text: "End notice now"; kind: "danger"; name: text + ": " + memberRow.person; confirmText: "Confirm end"
                                            effect: memberRow.running && memberRow.running.outcome === "remove" ? "They are removed from the site now." : "They become a steward now."
                                            onConfirmed: root.run("end_notice_now", [memberRow.modelData.key])
                                        }
                                        ConfirmButton {
                                            visible: root.isSuper && !!memberRow.running
                                            text: "Cancel notice"; name: text + ": " + memberRow.person; confirmText: "Confirm cancel"
                                            effect: "They stay an admin."
                                            onConfirmed: root.run("cancel_notice", [memberRow.modelData.key])
                                        }
                                        ActionButton {
                                            visible: root.isSuper && memberRow.sup && memberRow.own
                                            text: "Transfer super admin"
                                            onClicked: memberRow.panel = "transfer"
                                        }
                                        ConfirmButton {
                                            visible: root.isSuper && memberRow.sup && !memberRow.own
                                            text: "Remove super admin"; kind: "danger"; name: text + ": " + memberRow.person; confirmText: "Confirm remove"
                                            effect: "They become a regular admin. Immediate, no notice."
                                            onConfirmed: root.run("remove_super_admin", [memberRow.modelData.key])
                                        }
                                        ActionButton {
                                            visible: !memberRow.admin
                                            text: "Revoke"; kind: "danger"; Accessible.name: "Revoke " + memberRow.person
                                            onClicked: memberRow.panel = "revoke"
                                        }
                                    }
                                }
                                ReasonPanel {
                                    id: revokeStep
                                    visible: memberRow.panel === "revoke" && root.isAdmin
                                    Layout.leftMargin: 16 + 92 + 12
                                    title: "Revoke " + memberRow.person + "?"
                                    reasonName: "Reason for revoking " + memberRow.person
                                    effect: "They lose the " + memberRow.modelData.role + " role now. Everyone on the site can see the reason."
                                    okText: "Confirm revoke"
                                    onConfirmed: root.run("revoke_role", [memberRow.modelData.key, reason], function () { memberRow.panel = "" })
                                    onCancelled: memberRow.panel = ""
                                }
                                ReasonPanel {   // a notice: they stay an admin until the deadline, then the outcome applies by itself
                                    id: changeStep
                                    property int outcome: 0          // 0 steward, 1 remove
                                    property int span: 3             // index into `days`, or 4: the custom date and time
                                    readonly property var days: [0, 1, 7, 30]
                                    readonly property real deadline: end(root.now)
                                    // ponytail: "Now" is a minute ahead, so the core's later clock still sees a deadline to come.
                                    function end(t) {
                                        return span < 4 ? Math.floor(t) + Math.max(60, days[span] * 86400)
                                                        : Date.fromLocaleString(Qt.locale(), custom.text.trim(), "yyyy-MM-dd HH:mm").getTime() / 1000
                                    }
                                    visible: memberRow.panel === "change" && root.isSuper
                                    Layout.leftMargin: 16 + 92 + 12
                                    title: "Change admin: " + memberRow.person
                                    reasonName: "Reason for changing " + memberRow.person
                                    ready: deadline > Date.now() / 1000
                                    effect: !ready ? "Write a date and time to come, as 2026-11-11 14:00."
                                        : "They keep admin powers until " + root.when(deadline) + ". At that time they automatically "
                                          + (outcome ? "are removed from the site" : "become a steward")
                                          + ". Actions they sign after that time are rejected. Everyone can see the reason."
                                    okText: "Start notice"
                                    onConfirmed: root.run("admin_notice", [memberRow.modelData.key, outcome ? "remove" : "steward", Math.floor(end(Date.now() / 1000)), reason],
                                                          function () { memberRow.panel = "" })
                                    onCancelled: memberRow.panel = ""
                                    FormRow {
                                        label: "What happens at the end"; compact: true
                                        Segmented { name: "What happens at the end"; options: ["Move to steward", "Remove from site"]; current: changeStep.outcome; onPicked: function (i) { changeStep.outcome = i } }
                                    }
                                    FormRow {
                                        label: "Notice"; compact: true
                                        Segmented { name: "Notice"; options: ["Now", "1 day", "7 days", "30 days", "Custom"]; current: changeStep.span; onPicked: function (i) { changeStep.span = i } }
                                        FramedField {
                                            id: custom
                                            visible: changeStep.span === 4
                                            Layout.preferredWidth: 220
                                            placeholderText: "yyyy-MM-dd HH:mm"
                                            Accessible.name: "Notice ends at, as year-month-day hours:minutes"
                                        }
                                    }
                                }
                                ReasonPanel {   // hand our place to a named member, at once
                                    visible: memberRow.panel === "transfer" && root.isSuper
                                    Layout.leftMargin: 16 + 92 + 12
                                    title: "Transfer super admin"
                                    withReason: false
                                    ready: transferTo.currentIndex >= 0
                                    effect: "They become a super admin now. You become a regular admin. The change is immediate and everyone can see it."
                                    okText: "Transfer"
                                    onConfirmed: root.run("transfer_super_admin", [transferTo.currentValue], function () { memberRow.panel = "" })
                                    onCancelled: memberRow.panel = ""
                                    FormRow {
                                        label: "Pick a member"; compact: true
                                        FramedCombo {
                                            id: transferTo
                                            Layout.preferredWidth: 260
                                            textRole: "name"; valueRole: "key"
                                            model: root.members.filter(function (m) { return !!m.name && m.key !== root.me.key })
                                            Accessible.name: "Member to make super admin"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    CollapsibleSection {
                        title: "Removed"; aside: "· " + root.revoked.length
                        pad: 0
                        Blank {
                            visible: root.revoked.length === 0
                            icon: "res"; title: "No one has been removed."
                            text: root.isAdmin ? "When you revoke a role, the person shows here with your reason."
                                               : "When the admin removes someone's role, they show here with the reason."
                        }
                        Repeater {
                            model: root.revoked
                            delegate: GrantRow {
                                id: removedRow
                                again: true
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 10
                                    Fingerprint { text: removedRow.modelData.fingerprint }
                                    Label {
                                        Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.PlainText; font.weight: Font.DemiBold
                                        text: root.who(removedRow.modelData.key, removedRow.modelData.name)
                                    }
                                }
                                Help {
                                    text: "Removed by " + root.who(removedRow.modelData.by, removedRow.modelData.by_name) + " on "
                                          + root.when(removedRow.modelData.ts, "d MMM") + ": " + removedRow.modelData.reason
                                }
                            }
                        }
                    }
                }

                // Places (everyone in the site): the places and their change log, searchable. The admin
                // also adds, edits, retires and removes places here.
                TabPage {
                    id: placesPage
                    objectName: "placesPage"
                    maxWidth: 1040
                    enabled: !root.syncing
                    property bool adding: false    // "Add a place" was pressed: its form shows above the list
                    property bool fullLog: false    // the change log past its newest 20; forgotten once the page is left
                    onVisibleChanged: if (!visible) fullLog = false
                    readonly property int retiredCount: root.locations.filter(function (l) { return l.state === "retired" }).length
                    // The change log's search: any word of a row (code, name, person, action, reason), over the whole log.
                    readonly property string query: logSearch.text.trim().toLowerCase()
                    readonly property var logMatches: query === "" ? root.placeLog
                        : root.placeLog.filter(function (c) { return root.placeChange(c).toLowerCase().indexOf(query) >= 0 })
                    // A place's "Changes" link: its code in the search, then the log in view.
                    function showChanges(code) {
                        logSearch.text = code
                        changeLog.open = true
                        logSearch.forceActiveFocus()
                        Qt.callLater(function () {
                            var f = placesPage.contentItem
                            f.contentY = Math.max(0, Math.min(changeLog.mapToItem(f.contentItem, 0, 0).y - 16, f.contentHeight - f.height))
                        })
                    }
                    PageHead { title: "Places"; lede: root.isAdmin ? "Add, edit, retire and remove places, and see what changed." : "The places in this site, and what changed." }
                    CollapsibleSection {
                        title: "Places"
                        aside: "· " + root.locations.length + " places" + (placesPage.retiredCount > 0 ? ", " + placesPage.retiredCount + " retired" : "")
                        open: true; pad: 0
                        RowLayout {
                            Layout.fillWidth: true; Layout.margins: 16; Layout.bottomMargin: 8
                            spacing: 12
                            Help {
                                text: root.isAdmin
                                    ? "Nothing is deleted. A retired place takes no new reports; one with open issues can't be retired. "
                                      + "A place nobody ever reported can be removed: it is hidden after 30 days (undo until then), and its signed events stay in everyone's log. "
                                      + "Edit changes a place's name and group; its code never changes."
                                    : "Nothing is deleted. A retired place takes no new reports. A place marked \"hidden on\" a date will leave the lists then; its history stays. A code opens that place in History."
                            }
                            FramedButton {
                                visible: root.isAdmin
                                Layout.alignment: Qt.AlignTop
                                text: placesPage.adding ? "Cancel" : "Add a place"; iconName: placesPage.adding ? "x" : "plus"
                                onClicked: placesPage.adding = !placesPage.adding
                            }
                        }
                        ColumnLayout {
                            visible: placesPage.adding && root.isAdmin
                            Layout.fillWidth: true; Layout.margins: 16; Layout.topMargin: 0
                            spacing: 12
                            Help { text: "For a place found on the site walk. It joins the list straight away." }
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 16
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
                                spacing: 12
                                FormRow {
                                    label: "Code"
                                    Layout.fillWidth: false
                                    FramedField { id: locCode; Layout.preferredWidth: 140; maximumLength: 500; Accessible.name: "Code"; placeholderText: "e.g. W-04" }
                                }
                                FormRow {
                                    label: "Name"
                                    FramedField { id: locLabel; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Name"; placeholderText: "e.g. Tap behind tent 4"; onAccepted: addLoc.submit() }
                                }
                            }
                            RowLayout {
                                spacing: 12
                                ActionButton {
                                    id: addLoc
                                    text: "Add place"; kind: "primary"
                                    readonly property string group: locGroup.currentIndex === root.groupNames.length ? locNewGroup.text.trim() : locGroup.currentText
                                    readonly property string missing: group === "" ? "Name the new group." : locCode.text.trim() === "" ? "Type a code." : locLabel.text.trim() === "" ? "Type a name." : ""
                                    allowed: missing === ""
                                    onClicked: root.run("add_location", [locCode.text, locLabel.text, group],
                                                        function () { locCode.text = ""; locLabel.text = ""; locNewGroup.text = "" })
                                }
                                Help { visible: addLoc.missing !== ""; text: addLoc.missing }
                            }
                        }
                        PlaceBrowser {
                            bare: true
                            maxHeight: 900
                            listHeader: Component {
                                TableHead {
                                    PlaceCell { col: 0; muted: true; bold: true; text: "Code" }
                                    PlaceCell { col: 1; muted: true; bold: true; text: "Name" }
                                    PlaceCell { col: 2; muted: true; bold: true; text: "Group" }
                                    PlaceCell { col: 3; muted: true; bold: true; text: "Status" }
                                    PlaceCell { col: 4; muted: true; bold: true; align: Text.AlignRight; text: "Open issues" }
                                    PlaceCell { col: 5; muted: true; bold: true; text: "Log" }
                                    PlaceCell { col: 6; visible: root.isAdmin; muted: true; bold: true; text: "Actions" }
                                }
                            }
                            rowDelegate: Component {
                                Rectangle {
                                    id: locRow
                                    objectName: "placeRow-" + modelData.code
                                    required property var modelData
                                    readonly property string st: modelData.state
                                    readonly property bool pending: st === "pending_removal"
                                    // The core refuses a retire while issues are open; disabling is only a courtesy.
                                    readonly property bool blocked: st === "active" && modelData.open_issues > 0
                                    // What the row's inline panel is doing: "" (nothing), "edit", or the core method a confirm step
                                    // will run (each asks for a reason, which the core requires and every member can read).
                                    property string panel
                                    readonly property var ask: ({
                                        retire_location: { title: "Retire", ok: "Confirm retire", kind: "danger", effect: " takes no new reports. Its history stays, and you can restore it later." },
                                        restore_location: { title: "Restore", ok: "Confirm restore", kind: "primary", effect: " takes reports again." },
                                        remove_location: { title: "Remove", ok: "Confirm remove", kind: "danger", effect: " is hidden after 30 days. You can undo until then." },
                                        undo_remove_location: { title: "Undo the removal of", ok: "Confirm undo", kind: "primary", effect: " stays in the lists." } })[panel]
                                    onPanelChanged: confirmStep.reset()
                                    width: ListView.view.width
                                    height: stack.implicitHeight + 1
                                    color: panel !== "" ? root.t.accentBg : st === "active" ? root.t.card : root.t.well
                                    Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: root.t.bd }
                                    ColumnLayout {
                                        id: stack
                                        width: parent.width
                                        spacing: 0
                                        RowLayout {
                                            Layout.fillWidth: true; Layout.margins: 10; Layout.leftMargin: 16; Layout.rightMargin: 16
                                            spacing: 12
                                            PlaceCell { col: 0; PlaceLink { code: locRow.modelData.code } }
                                            PlaceCell {
                                                col: 1; text: locRow.modelData.label; ink: locRow.st === "active" ? root.t.fg : root.t.muted
                                                Label {
                                                    visible: !!locRow.modelData.renamed_from
                                                    width: parent.width; wrapMode: Text.Wrap; textFormat: Text.PlainText
                                                    font.pointSize: root.smallSize; color: root.t.muted
                                                    text: "was “" + locRow.modelData.renamed_from + "”"
                                                }
                                            }
                                            PlaceCell { col: 2; muted: true; text: locRow.modelData.group }
                                            PlaceCell {
                                                col: 3
                                                StatusChip {
                                                    visible: !locRow.pending
                                                    label: locRow.st === "retired" ? "Retired" : "Active"
                                                    stage: locRow.st === "retired" ? root.t.closed : root.t.resolved
                                                }
                                                Label {
                                                    visible: text !== ""
                                                    width: parent.width; wrapMode: Text.Wrap; textFormat: Text.PlainText
                                                    color: locRow.pending ? root.t.danger : root.t.muted
                                                    font.pointSize: root.smallSize; font.weight: locRow.pending ? Font.DemiBold : Font.Normal
                                                    text: locRow.pending ? "hidden on " + root.when(locRow.modelData.removes_at, "d MMM") + ": " + locRow.modelData.removal_reason
                                                        : locRow.st === "retired" ? locRow.modelData.retired_reason : ""
                                                }
                                                Flow {   // the Open issues and Log columns, once they have dropped out
                                                    visible: root.placeTable.compact
                                                    width: parent.width; spacing: 8
                                                    Label {
                                                        visible: locRow.modelData.open_issues > 0
                                                        font.pointSize: root.smallSize; color: root.t.muted; text: root.openIssuesText(locRow.modelData.open_issues)
                                                    }
                                                    ChangesLink { code: locRow.modelData.code; onClicked: placesPage.showChanges(code) }
                                                }
                                            }
                                            PlaceCell {
                                                col: 4; align: Text.AlignRight; text: locRow.modelData.open_issues
                                                ink: locRow.modelData.open_issues > 0 ? root.t.fg : root.t.muted
                                            }
                                            PlaceCell { col: 5; ChangesLink { code: locRow.modelData.code; onClicked: placesPage.showChanges(code) } }
                                            PlaceCell {   // three fixed slots, so every row's buttons line up
                                                col: 6; visible: root.isAdmin
                                                RowLayout {
                                                    width: parent.width; spacing: 6
                                                    ActionButton {   // slot 1: Retire / Restore
                                                        shown: !locRow.pending
                                                        Layout.preferredWidth: 80
                                                        text: locRow.st === "retired" ? "Restore" : "Retire"
                                                        allowed: !locRow.blocked
                                                        Accessible.name: text + " " + locRow.modelData.code
                                                        onClicked: locRow.panel = locRow.st === "retired" ? "restore_location" : "retire_location"
                                                    }
                                                    ActionButton {   // slot 2: Remove / Undo removal; only for a place no report ever named, the core decides
                                                        shown: locRow.pending || !locRow.modelData.ever_used
                                                        Layout.preferredWidth: 124
                                                        kind: locRow.pending ? "default" : "danger"
                                                        text: locRow.pending ? "Undo removal" : "Remove"
                                                        Accessible.name: text + " " + locRow.modelData.code
                                                        onClicked: locRow.panel = locRow.pending ? "undo_remove_location" : "remove_location"
                                                    }
                                                    ActionButton {   // slot 3: Edit
                                                        shown: !locRow.pending
                                                        Layout.preferredWidth: 76
                                                        text: locRow.panel === "edit" ? "Cancel" : "Edit"
                                                        Accessible.name: (locRow.panel === "edit" ? "Cancel editing " : "Edit ") + locRow.modelData.code
                                                        onClicked: locRow.panel = locRow.panel === "edit" ? "" : "edit"
                                                    }
                                                }
                                                Label {
                                                    visible: locRow.blocked
                                                    width: parent.width; wrapMode: Text.Wrap
                                                    font.pointSize: root.smallSize; color: root.t.muted; text: "Close its issues first to retire it."
                                                }
                                            }
                                        }
                                        ReasonPanel {   // the confirm step: under the name, with the reason the core needs
                                            id: confirmStep
                                            visible: !!locRow.ask && root.isAdmin
                                            Layout.leftMargin: 16 + root.placeTable.w[0] + 12
                                            title: locRow.ask ? locRow.ask.title + " " + locRow.modelData.code + ", " + locRow.modelData.label + "?" : ""
                                            reasonName: "Reason for changing " + locRow.modelData.code
                                            effect: locRow.ask ? locRow.modelData.label + locRow.ask.effect : ""
                                            okText: locRow.ask ? locRow.ask.ok : ""; kind: locRow.ask ? locRow.ask.kind : "default"
                                            onConfirmed: root.run(locRow.panel, [locRow.modelData.code, reason], function () { locRow.panel = "" })
                                            onCancelled: locRow.panel = ""
                                        }
                                        RowLayout {   // the code is the place's identity: only name and group change
                                            visible: locRow.panel === "edit" && root.isAdmin
                                            Layout.fillWidth: true
                                            Layout.leftMargin: 16; Layout.rightMargin: 16; Layout.bottomMargin: 12
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
                                                text: "Save"; kind: "primary"
                                                allowed: editLabel.text.trim() !== "" && editGroup.text.trim() !== ""
                                                onClicked: root.run("edit_location", [locRow.modelData.code, editLabel.text, editGroup.text],
                                                                    function () { locRow.panel = "" })
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    CollapsibleSection {
                        id: changeLog
                        objectName: "changeLog"
                        title: "Change log"
                        aside: "· " + (placesPage.query ? placesPage.logMatches.length + " of " : "") + root.placeLog.length
                        pad: 0
                        Help {
                            Layout.margins: 16; Layout.bottomMargin: 8
                            text: "Removed places are hidden from every list, retired ones included. Removed means hidden: the signed events stay in everyone's log."
                        }
                        Help {
                            visible: root.placeLog.length === 0
                            Layout.margins: 16; Layout.topMargin: 0
                            text: "No changes yet."
                        }
                        FormRow {
                            visible: root.placeLog.length > 0
                            Layout.margins: 16; Layout.topMargin: 4; Layout.bottomMargin: 12
                            label: "Search"; compact: true
                            FramedField {
                                id: logSearch
                                objectName: "logSearch"
                                Layout.fillWidth: true
                                placeholderText: "place, person, action (retired, removed, renamed…) or reason"
                                Accessible.name: "Search the change log"
                            }
                        }
                        Blank {
                            visible: placesPage.query !== "" && placesPage.logMatches.length === 0
                            Layout.topMargin: 8
                            icon: "search"; title: "No changes match “" + logSearch.text.trim() + "”."
                            text: "Search by place code or name, a person, an action or a reason."
                            FramedButton { text: "Clear"; Accessible.name: "Clear the change log search"; onClicked: logSearch.text = "" }
                        }
                        TableHead {
                            visible: placesPage.logMatches.length > 0
                            LogCell { col: 0; muted: true; bold: true; text: "When" }
                            LogCell { col: 1; muted: true; bold: true; text: "Place" }
                            LogCell { col: 2; muted: true; bold: true; text: "Action" }
                            LogCell { col: 3; muted: true; bold: true; text: "By" }
                            LogCell { col: 4; muted: true; bold: true; text: "Reason / detail" }
                        }
                        Repeater {
                            model: placesPage.fullLog || placesPage.query ? placesPage.logMatches : root.placeLog.slice(0, 20)
                            delegate: LogRow {}
                        }
                        MoreToggle {
                            visible: !placesPage.query && root.placeLog.length > 20
                            Layout.fillWidth: true; Layout.margins: 16
                            expanded: placesPage.fullLog
                            text: expanded ? "Show the newest 20" : "+" + root.plural(root.placeLog.length - 20, "older change")
                            onClicked: placesPage.fullLog = !placesPage.fullLog
                        }
                    }
                }

                // Integrity (from History's strip and the Board's mismatch banner): the record's status, then the three
                // anchor steps for any member: compute, run the printed spel command, record the reference
                TabPage {
                    enabled: !root.syncing
                    PageHead {
                        title: "Integrity"
                        lede: "Anyone can check that the record was not changed after the fact. An anchor writes a fingerprint of everyone's "
                            + "history to the Logos blockchain (LEZ), so no one, not even the admin, can quietly rewrite or delete past events."
                    }
                    Note { text: root.plural(root.info.events || 0, "event") + " · " + root.conflicts; font.weight: Font.DemiBold }
                    Flash {   // the newest anchor: one state at a time (root.anchorState)
                        readonly property var a: root.lastAnchor
                        err: root.anchorState === "mismatch"
                        tone: err ? "err" : root.anchorState === "" ? "plain" : "warn"
                        icon: tone === "plain" ? "anchor" : "warn"
                        lead: err ? "This anchor doesn't match the record on this device." : ""
                        text: ({ mismatch: "If it is still here after this device syncs, tell your admin.",
                                 never: "Nothing has been anchored yet. Until then, history can be changed without anyone noticing.",
                                 stale: a ? "Last anchor was " + root.age(a.ts) + ". Any member can anchor from this page." : "",
                                 "": a ? root.anchorSummary(a) : "" })[root.anchorState]
                    }
                    Help { visible: !root.approved; text: "Members can anchor the record. After approval you can do this too." }
                    Heading { visible: root.approved; text: "Anchor now" }
                    Help { visible: root.approved; text: "Needs a terminal and a LEZ account." }
                    Step {
                        visible: root.approved
                        n: 1; title: "Compute the checkpoint"
                        ActionButton { text: "Compute checkpoint"; kind: "primary"; onClicked: root.run("checkpoint_now", [], function (r) { if (r && r.heads_root) root.checkpoint = r }) }
                        Label {
                            visible: !!root.checkpoint; Layout.topMargin: 2; textFormat: Text.RichText
                            text: root.checkpoint ? root.checkpoint.n_events + " events, root <span style='font-family:monospace'>" + root.esc(root.checkpoint.heads_root.substr(0, 16)) + "…</span>" : ""
                        }
                    }
                    Step {
                        visible: root.approved
                        n: 2; title: "Copy the command"
                        Help { text: "Run it in programs/pukaar_registry/ in a terminal. Replace <YOUR_PUBLIC_ACCOUNT> with your LEZ account." }
                        FramedTextArea {
                            readOnly: true; mono: true; boxHeight: 112
                            name: "spel anchor command"
                            placeholder: "Compute the checkpoint first"
                            text: root.checkpoint ? root.checkpoint.spel : ""
                        }
                    }
                    Step {
                        visible: root.approved
                        n: 3; title: "Record the reference"
                        Help { text: "Paste the reference the command prints, then record it." }
                        LabelledField {
                            id: anchorField
                            label: "Reference"
                            placeholder: "e.g. 0x9f2c…e1 or pda:<account id>"
                            mono: true
                            buttonText: "Record"
                            missing: !root.checkpoint ? "Compute the checkpoint first (step 1)." : anchorField.text.trim() === "" ? "Paste the reference first." : ""
                            onSubmitted: {
                                root.run("record_anchor", [JSON.stringify(root.checkpoint.heads), anchorField.text],
                                         function () { anchorField.text = ""; root.checkpoint = null })
                            }
                        }
                    }
                }

                // Profile (the you block opens it): me on this device. Name, backup, invite, leave.
                TabPage {
                    id: profilePage
                    readonly property string myName: root.me.name || ""   // a string: changes only when the name does, not on every refresh
                    readonly property var myReports: root.issues.filter(function (i) { return i.reporter === root.me.key })
                    readonly property int mineOpen: myReports.filter(function (i) { return !root.isTerminal(i.status) }).length
                    PageHead { title: "You"; lede: root.esc(root.me.name || "pseudonym") + " · " + root.roleName }
                    Flash {
                        visible: root.isSuper && root.superCount === 1
                        Accessible.role: Accessible.AlertMessage
                        err: true
                        lead: "Back up your key now."
                        text: "You are the only super admin. If you lose this key, nobody can manage admins. Add a second super admin or back up your key."
                        buttonText: "Back up key"
                        onActivated: exportPassword.forceActiveFocus()
                    }
                    FormRow {
                        label: "Your code"; sub: root.unapproved ? "(read it aloud at the kiosk)" : ""
                        Fingerprint { text: root.me.fingerprint || ""; big: root.unapproved }
                    }
                    Heading { text: "Name" }
                    Note { visible: root.staff; text: root.me.name || ""; font.weight: Font.DemiBold }
                    Help { visible: root.staff; text: "Staff names are set by the admin when granting the role." }
                    LabelledField {
                        id: nameField
                        visible: !root.staff
                        label: "Display name"
                        text: profilePage.myName
                        placeholder: "e.g. Asha (or leave empty to stay pseudonymous)"
                        buttonText: "Set name"
                        onSubmitted: root.run("set_profile", [nameField.text])
                    }
                    Help {
                        visible: !root.staff
                        text: "The admin may have named you when approving you; this replaces it. Set it empty to stay a pseudonym; the admin who granted your role can still link it. Staff are always named."
                    }
                    Help { visible: root.removed; text: "Setting your name asks to join again: you then wait for the admin's approval." }
                    Heading { text: "Back up this identity" }
                    Help { text: "Your identity is a key on this device. A backup lets you continue as the same person after a reinstall." }
                    Flash {
                        err: true
                        lead: "Keep this and your password safe."
                        text: "Anyone with both can act as you. Never run the same identity on two devices at once."
                    }
                    FormRow {
                        label: "Password"; sub: "(at least 8 characters)"
                        FramedField { id: exportPassword; Layout.fillWidth: true; echoMode: TextInput.Password; Accessible.name: "Backup password"; placeholderText: "e.g. a long phrase only you know" }
                    }
                    LabelledField {
                        id: exportConfirm
                        label: "Repeat the password"
                        placeholder: "type the same password again"
                        echoMode: TextInput.Password
                        buttonText: "Export identity"; buttonKind: "primary"
                        missing: exportPassword.text.length < 8 ? "The password needs at least 8 characters."
                               : exportConfirm.text !== exportPassword.text ? "Repeat the same password." : ""
                        onSubmitted: {
                            root.run("export_identity", [exportPassword.text],
                                     function (r) { exportOut.text = r; exportPassword.text = ""; exportConfirm.text = "" })
                        }
                    }
                    FormRow {
                        visible: exportOut.text !== ""
                        label: "Identity backup"
                        FramedTextArea {
                            id: exportOut
                            readOnly: true; mono: true; boxHeight: 90
                            name: "identity backup"
                            FramedButton { text: "Hide"; onClicked: exportOut.text = "" }
                        }
                    }
                    Heading { visible: root.approved; text: "Invite someone" }
                    Help { visible: root.approved; text: "People join with this site id. An admin approves them after they read their code aloud." }
                    RowLayout {
                        visible: root.approved
                        spacing: 8
                        Label { text: "Site id  " + root.shortId(root.me.site || ""); font.family: "monospace" }
                        FramedButton { text: "Copy"; iconName: "copy"; Accessible.name: "Copy the full site id"; onClicked: root.copyText(root.me.site) }
                    }
                    Heading { visible: root.approved; text: "My reports" }
                    RowLayout {
                        visible: root.approved
                        spacing: 12
                        Label { text: profilePage.mineOpen + " open · " + (profilePage.myReports.length - profilePage.mineOpen) + " closed" }
                        LinkButton { text: "Show on the Board  ›"; onClicked: { root.onlyMine = true; root.openPage("board") } }
                    }
                    Heading { text: "Leave this site" }
                    Help { text: "Removes this device's copy of the site. Your key stays." }
                    ConfirmButton {
                        text: "Leave this site…"; kind: "danger"; confirmText: "Confirm leave"
                        effect: "You go back to the start screen."
                        onConfirmed: root.leave()
                    }
                }

                // History (NOTES.md section 8): every resolved and closed issue, newest-closed first,
                // filtered. Nothing is deleted; an old issue opens in the same pane, with Still broken? Reopen.
                TabPage {
                    id: hist
                    maxWidth: 100000
                    readonly property int pageSize: 40
                    property int limit: pageSize
                    property int status: 0        // All, Resolved, Closed without fix, Reopened ≥1×
                    property int since: 3         // 7 days, 30 days, 3 months, All
                    property bool oldestFirst: false   // the Closed header's sort
                    readonly property string place: root.historyPlace
                    readonly property var all: root.issues.filter(function (i) { return root.isTerminal(i.status) }).sort(function (a, b) { return (oldestFirst ? -1 : 1) * root.newestClosedFirst(a, b) })
                    readonly property var atPlace: place === "" ? all : all.filter(function (i) { return i.location === place })
                    readonly property var shown: {
                        var q = search.text.trim().toLowerCase(), days = [7, 30, 91, 0][since], now = Date.now() / 1000
                        return atPlace.filter(function (i) {
                            return root.mineOk(i)
                                && (status === 0 || (status === 1 ? i.status === "ConfirmedResolved" : status === 2 ? i.status !== "ConfirmedResolved" : i.reopen_count > 0))
                                && (days === 0 || now - root.closedAt(i) < days * 86400)
                                && (q === "" || [i.text, i.location, i.location_label, i.landmark, i.category].join(" ").toLowerCase().indexOf(q) >= 0)
                        })
                    }
                    // the page's rows, each with its month for the section headers
                    readonly property var rows: shown.slice(0, limit).map(function (i) { return Object.assign({ month: root.when(root.closedAt(i), "MMMM yyyy") }, i) })
                    readonly property string filters: [search.text, place, status, since, root.onlyMine, oldestFirst].join("|")
                    onFiltersChanged: limit = pageSize
                    function count(l, f) { return l.filter(f).length }
                    function clear() { search.text = ""; status = 0; since = 3; root.onlyMine = false; root.historyPlace = "" }
                    // "6 issues · fixed within target: 6 of 6" (the target counts fixes the core saw claimed)
                    function monthAside(month) {
                        var m = shown.filter(function (i) { return root.when(root.closedAt(i), "MMMM yyyy") === month })
                        var claimed = m.filter(function (i) { return !!i.claimed_ts })
                        return root.plural(m.length, "issue") + (claimed.length ? " · fixed within target: " + count(claimed, root.inTarget) + " of " + claimed.length : "")
                    }
                    // "Confirmed: works (Asha)": the closing note and who closed it
                    function closingNote(i) {
                        var p = i.progress
                        return p && p.note ? p.note + " (" + root.who(p.by, p.by_name) + ")" : ""
                    }
                    // "Resolved 4 Oct · Confirmed: works (Asha) · reported by you · still on the board" (for screen readers)
                    function meta(i) {
                        return root.closedVerb(i) + " " + root.when(root.closedAt(i), "d MMM") + (closingNote(i) ? " · " + closingNote(i) : "")
                             + (i.reporter === root.me.key ? " · reported by you" : "") + (root.onBoard(i) ? " · still on the board" : "")
                    }
                    // the place view: "14 fixed, 2 reopened, 1 won't fix, 2 duplicates since 22 May. Open now: 1, on the board."
                    readonly property string placeSummary: {
                        var at = root.issues.filter(function (i) { return i.location === place })
                        var n = function (st) { return count(at, function (i) { return i.status === st }) }
                        var parts = [n("ConfirmedResolved") + " fixed"], reopened = count(at, function (i) { return i.reopen_count > 0 })
                        if (reopened) parts.push(reopened + " reopened")
                        if (n("ClosedWontfix")) parts.push(n("ClosedWontfix") + " won't fix")
                        if (n("Duplicate")) parts.push(root.plural(n("Duplicate"), "duplicate"))
                        var open = count(at, function (i) { return !root.isTerminal(i.status) })
                        return parts.join(", ") + (at.length ? " since " + root.when(Math.min.apply(null, at.map(function (i) { return i.reported_ts })), "d MMM") : "")
                             + ". Open now: " + open + (open ? ", on the board." : ".")
                    }
                    IntegrityStrip {}
                    PageHead {
                        title: "History"
                        lede: hist.place ? "Everything reported at one place. <b>Nothing is deleted.</b>"
                            : "Every resolved and closed issue, " + (hist.oldestFirst ? "oldest" : "newest") + " first. The board shows resolved issues for 14 days and closed ones for 30; "
                              + "after that they are only here. <b>Nothing is deleted</b>, and any of them can be reopened."
                    }
                    RowLayout {
                        Layout.fillWidth: true; Layout.topMargin: 8
                        spacing: 12
                        FormRow {
                            label: "Search"; compact: true
                            FramedField { id: search; Layout.fillWidth: true; placeholderText: "e.g. tap, नल, or a place code"; Accessible.name: "Search History" }
                        }
                        FormRow {
                            label: "Place"; compact: true
                            Layout.fillWidth: false
                            FramedCombo {   // a card's place link sets the filter too, so the shown text follows root.historyPlace
                                Layout.preferredWidth: 200
                                Accessible.name: "Filter by place"
                                textRole: "label"; valueRole: "code"
                                model: [{ code: "", label: "All places" }]
                                    .concat(root.locations.map(function (l) { return { code: l.code, label: l.code + " " + l.label } }))
                                    .concat([{ code: "other", label: "Other (not on the list)" }])
                                displayText: hist.place === "" ? "All places" : hist.place === "other" ? "Other" : root.placeLabel(hist.place)
                                onActivated: function (index) { root.historyPlace = valueAt(index) }
                            }
                        }
                    }
                    Flow {
                        Layout.fillWidth: true; Layout.bottomMargin: 4
                        spacing: 12
                        FormRow {
                            id: statusRow
                            label: "Status"; compact: true
                            Segmented { name: "Status"; options: ["All", "Resolved", "Closed without fix", "Reopened ≥1×"]; current: hist.status; onPicked: function (i) { hist.status = i } }
                        }
                        FormRow {
                            label: "When"; compact: true
                            Segmented { name: "When"; options: ["7 days", "30 days", "3 months", "All"]; current: hist.since; onPicked: function (i) { hist.since = i } }
                        }
                        Item {   // on the segments' line, level with their buttons; wrapped, just the box
                            width: mine.implicitWidth; height: y > 0 ? mine.implicitHeight : statusRow.height
                            MineCheck { id: mine; anchors.bottom: parent.bottom }
                        }
                    }
                    Rectangle { Layout.fillWidth: true; implicitHeight: 1; color: root.t.bd }
                    Rectangle {   // the place view: what happened here, and the way back to every place
                        visible: hist.place !== ""
                        Layout.fillWidth: true; Layout.topMargin: 4
                        implicitHeight: psum.implicitHeight + 20
                        radius: 6; color: root.t.card; border.color: root.t.bd
                        ColumnLayout {
                            id: psum
                            x: 12; y: 10; width: parent.width - 24
                            spacing: 6
                            Label {
                                Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.StyledText
                                text: "<b>" + root.esc(hist.place === "other" ? "Other places" : root.placeLabel(hist.place)) + ":</b> " + root.esc(hist.placeSummary)
                            }
                            Row {
                                spacing: 10
                                StatusChip { label: "Place: " + (hist.place === "other" ? "Other" : hist.place); stage: root.t.progress }
                                LinkButton { anchors.verticalCenter: parent.verticalCenter; text: "Clear place"; onClicked: root.historyPlace = "" }
                            }
                        }
                    }
                    Label {   // the count line
                        visible: hist.all.length > 0
                        Layout.fillWidth: true; wrapMode: Text.Wrap; textFormat: Text.StyledText
                        font.pointSize: root.smallSize; color: root.t.muted
                        readonly property string fg: "<font color='" + root.t.fg + "'>"
                        readonly property string showing: hist.rows.length < hist.shown.length ? " · showing " + hist.rows.length : ""
                        text: hist.place ? "<b>" + fg + root.plural(hist.atPlace.length, "closed issue") + "</font></b> at " + (hist.place === "other" ? "Other" : hist.place)
                                           + (hist.shown.length < hist.atPlace.length ? " · " + hist.shown.length + " match" : "") + showing
                            : hist.shown.length < hist.all.length ? "<b>" + fg + hist.shown.length + " of " + root.plural(hist.all.length, "issue") + "</font></b>" + showing
                            : "<b>" + fg + root.plural(hist.all.length, "issue") + "</font></b> · " + hist.count(hist.all, function (i) { return i.status === "ConfirmedResolved" }) + " resolved · "
                              + hist.count(hist.all, function (i) { return i.status !== "ConfirmedResolved" }) + " closed without fix · "
                              + hist.count(hist.all, function (i) { return i.reopen_count > 0 }) + " reopened at least once" + showing
                    }
                    Blank {
                        visible: hist.all.length === 0
                        icon: "hist"; title: "Nothing resolved or closed yet"
                        text: "Issues are listed here once they are resolved or closed, for good. Nothing is deleted."
                    }
                    Blank {
                        visible: hist.all.length > 0 && hist.shown.length === 0
                        icon: "search"; title: "No issues match"
                        text: "Nothing" + (root.onlyMine ? " you reported" : "") + (hist.since < 3 ? " in the last " + ["7 days", "30 days", "3 months"][hist.since] : "")
                            + (search.text.trim() ? " matches “" + search.text.trim() + "”" : " matches these filters") + ". "
                            + (hist.since < 3 ? "Try All dates, or clear the search. " : "") + "Nothing is deleted, so if it was reported, it is in here."
                        FramedButton { text: "Clear filters"; onClicked: hist.clear() }
                    }
                    Rectangle {   // the list: month headers, then rows; inset 16 px, so the sidebar toggle never touches a row
                        visible: hist.rows.length > 0
                        Layout.fillWidth: true; Layout.topMargin: 4; Layout.bottomMargin: 16
                        implicitHeight: histList.contentHeight + 2
                        radius: 8; color: root.t.card; border.color: root.t.bd
                        ListView {
                            id: histList
                            property int pad: 16            // row inset, read by SectionHeader
                            x: 1; y: 1; width: parent.width - 2; height: contentHeight
                            interactive: false
                            model: hist.rows
                            header: TableHead {   // the column titles; only Closed sorts
                                width: histList.width
                                HistCell {
                                    col: 0
                                    AbstractButton {
                                        id: sort
                                        text: "Closed " + (hist.oldestFirst ? "▲" : "▼")
                                        focusPolicy: Qt.StrongFocus
                                        Accessible.name: "Sort by closed date, " + (hist.oldestFirst ? "oldest" : "newest") + " first"
                                        onClicked: hist.oldestFirst = !hist.oldestFirst
                                        Keys.onReturnPressed: clicked()
                                        Keys.onEnterPressed: clicked()
                                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                                        contentItem: Label {
                                            text: sort.text; font.pointSize: root.smallSize; font.weight: Font.DemiBold
                                            color: sort.hovered ? root.t.fg : root.t.muted
                                        }
                                        background: Rectangle { anchors { fill: parent; margins: -3 } radius: 4; color: "transparent"; border.width: sort.visualFocus ? 2 : 0; border.color: root.t.focus }
                                    }
                                }
                                HistCell { col: 1; muted: true; bold: true; text: "Place" }
                                HistCell { col: 2; muted: true; bold: true; text: "Issue" }
                                HistCell { col: 3; muted: true; bold: true; text: "Status" }
                                HistCell { col: 4; muted: true; bold: true; text: "Category" }
                                HistCell { col: 5; muted: true; bold: true; text: "Closing note" }
                                HistCell { col: 6; muted: true; bold: true; text: "Reported by" }
                                HistCell { col: 7; muted: true; bold: true; align: Text.AlignRight; text: "Took" }
                            }
                            section.property: "month"
                            section.delegate: SectionHeader { aside: hist.monthAside(section) }
                            delegate: HistRow {
                                required property var modelData
                                issue: modelData; hint: hist.meta(modelData); note: hist.closingNote(modelData)
                                onClicked: root.openIssue(modelData.id)
                            }
                            footer: Item {
                                readonly property int remaining: hist.shown.length - hist.rows.length
                                width: histList.width; height: remaining > 0 ? 61 : 0
                                FramedButton {
                                    visible: parent.remaining > 0; anchors.centerIn: parent
                                    text: "Show " + Math.min(hist.pageSize, parent.remaining) + " more"
                                    onClicked: hist.limit += hist.pageSize
                                }
                            }
                        }
                    }
                }
            }

            // The issue pane: header, the report, why it isn't closed yet, then the actions grouped by
            // who they are for, what isn't available (and why), and the timeline last. It scrolls whole.
            ScrollView {
                id: detail
                visible: root.paneOpen
                Layout.fillWidth: root.narrow   // wide: a fixed pane beside the page; narrow: it takes over
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
                readonly property bool finished: root.isTerminal(st)   // resolved or closed: Still broken? Reopen
                Keys.onEscapePressed: root.closeIssue()
                property bool hasNote: note.text.trim() !== ""   // the core rejects these without one
                readonly property string issueId: issue.id || ""
                onIssueIdChanged: { wontFix.armed = false; markDup.armed = false }   // a confirm never carries over to another issue
                readonly property int etaHours: parseInt(eta.text) || 0
                function act(a, eta) {
                    root.run("act", [issue.id, a, note.text, nextStep.text, eta | 0],
                             function () { note.text = ""; nextStep.text = "" })
                }
                // The reporter brings it straight back to Reported (and the board shows it there); another
                // resident's reopen is one of two votes, so it may stay where it is.
                function reopenOld() {
                    var i = issue, code = root.placeCode(i)
                    root.run("act", [i.id, "reopen", reopenReason.text, "", 0], function () {
                        reopenReason.text = ""
                        if (i.reporter !== root.me.key) {
                            root.notice = "Your reopen of " + code + " is recorded. It goes back on the board once a second resident, or the reporter, reopens it too."
                            return
                        }
                        root.notice = code + " is back on the board, in Reported. It carries its whole history and shows Reopened " + (i.reopen_count + 1) + "×."
                        root.openPage("board")
                    })
                }
                ColumnLayout {
                    x: (detail.availableWidth - width) / 2
                    width: Math.min(detail.availableWidth, 760)
                    spacing: 0
                    RowLayout {
                        Layout.fillWidth: true; Layout.bottomMargin: 8
                        Item { visible: !root.narrow; Layout.fillWidth: true }
                        FramedButton {   // wide: an icon-only ×; narrow: back to the page underneath ("Close" reads like an outcome)
                            id: paneBack
                            readonly property string pageName: root.page === "history" ? "History" : "Board"
                            text: root.narrow ? "Back" : ""; iconName: root.narrow ? "chevl" : "x"
                            Accessible.name: root.narrow ? "Back to " + pageName : "Close panel"
                            onClicked: root.closeIssue()
                            SideTip { visible: !root.narrow && (paneBack.hovered || paneBack.visualFocus); text: "Close panel"; x: parent.width - width; y: parent.height + 6 }
                        }
                    }
                    Flow {   // code, stage, reopens, the reporter's call to action, category
                        Layout.fillWidth: true; Layout.bottomMargin: 4
                        spacing: 6
                        Item {   // the code: a link to that place's History ("Other" has no page of its own)
                        width: other ? otherCode.implicitWidth : paneCode.implicitWidth; height: 24
                        readonly property bool other: detail.issue.location === "other"
                        Label { id: otherCode; visible: parent.other; height: 24; verticalAlignment: Text.AlignVCenter; textFormat: Text.RichText; text: root.mono("Other") }
                        PlaceLink { id: paneCode; visible: !parent.other; anchors.verticalCenter: parent.verticalCenter; code: detail.issue.location || "" }
                    }
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
                    Flash {   // an old issue: why it is not on the board, and that nothing is lost
                        visible: detail.finished && !root.onBoard(detail.issue)
                        Layout.topMargin: 8; Layout.bottomMargin: 4
                        text: visible ? root.offBoardText(detail.issue) : ""
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
                                      + root.mono(String(detail.issue.reporter).substr(0, 6)) + root.formerTag(detail.issue.former) + " · " + root.when(detail.issue.reported_ts)
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
                        readonly property var plan: detail.finished ? root.closedCard(detail.issue) : root.whyCard(detail.issue)
                        visible: plan !== null
                        Layout.fillWidth: true; Layout.bottomMargin: 16
                        card: plan || ({ intro: "", steps: [], never: "" }); stage: detail.stage
                        title: detail.finished ? "How it was closed" : "Why it isn't closed yet"
                        icon: detail.finished ? root.stageIcon(detail.st) : "info"
                    }
                    ActionGroup {
                        // the reporter decides alone; another resident's confirm or reopen is one of two votes
                        title: detail.mine ? "For the person who reported it" : "For residents"
                        visible: detail.canConfirm
                        enabled: !root.syncing
                        Layout.bottomMargin: 14
                        ActionButton {
                            kind: "primary"
                            text: detail.mine ? "Confirm it's fixed" : "Confirm (" + detail.issue.confirms + " of 2 residents so far)"
                            onClicked: detail.act("confirm", 0)
                        }
                    }
                    Rectangle {   // a claimed fix or a finished issue: reopen it, with its own reason, rather than report it again
                        visible: detail.canReopen
                        Layout.fillWidth: true; Layout.bottomMargin: 14
                        implicitHeight: stillCol.implicitHeight + 24
                        radius: 8; color: root.t.card; border.color: root.t.ctlBd
                        Accessible.role: Accessible.Grouping; Accessible.name: "Still broken? Reopen"
                        ColumnLayout {
                            id: stillCol
                            x: 12; y: 12; width: parent.width - 24
                            spacing: 6
                            RowLayout {
                                spacing: 8
                                Icon { name: "reopen"; size: 16; color: root.t.accent }
                                Label { Layout.fillWidth: true; wrapMode: Text.Wrap; text: "Still broken? Reopen"; font.weight: Font.DemiBold; font.pointSize: root.baseSize * 15 / 14 }
                            }
                            Help {
                                text: "Reopen this issue rather than reporting a new one. It goes back to Reported with its whole history and shows Reopened "
                                    + ((detail.issue.reopen_count || 0) + 1) + "×."
                            }
                            FormRow {
                                label: "Reason"; sub: "(what is wrong now)"
                                FramedTextArea {
                                    id: reopenReason
                                    name: "Reason for reopening"; boxHeight: 60; maxLength: 500
                                    wrap: TextEdit.WrapAtWordBoundaryOrAnywhere
                                    placeholder: "e.g. Leaking again under the tap"
                                }
                            }
                            ActionButton {
                                kind: detail.canConfirm ? "default" : "primary"
                                allowed: reopenReason.text.trim() !== ""
                                text: "Reopen (reason)" + (detail.issue.reopen_count > 0 ? ", reopened " + detail.issue.reopen_count + "×" : "")
                                onClicked: detail.reopenOld()
                            }
                        }
                    }
                    FormRow {
                        visible: root.approved
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
                            label: "ETA (hours)"
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
                            FramedButton { id: more; checkable: true; text: "More"; iconName: checked ? "chevd" : "chev" }   // close without a fix
                            ConfirmButton {
                                id: wontFix
                                visible: more.checked; allowed: detail.hasNote; kind: "danger"; text: "Won't fix (reason)"; confirmText: "Confirm won't fix"
                                effect: "Closes it without a fix."
                                onConfirmed: detail.act("close_wontfix", 0)
                            }
                            RowLayout {   // close this one as a duplicate of another open issue
                                visible: more.checked
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
                                ConfirmButton {
                                    id: markDup
                                    text: "Mark duplicate"; confirmText: "Confirm duplicate"
                                    effect: "Closes it in favour of the other."
                                    allowed: dupOf.target !== ""
                                    onConfirmed: root.run("act", [detail.issue.id, "mark_duplicate", dupOf.target, "", 0],
                                                          function () { dupOf.target = "" })
                                }
                            }
                        }
                        ActionGroup {
                            title: "For everyone"
                            visible: root.approved
                            ActionButton { id: comment; allowed: detail.hasNote; text: "Comment"; onClicked: detail.act("comment", 0) }
                        }
                    }
                    Rectangle {   // what this viewer can't do here, and why (the same rules as above)
                        id: na
                        readonly property var lines: root.unavailable(detail.issue, detail.hasNote)
                        visible: root.approved && (lines.length > 0 || detail.finished)
                        Layout.fillWidth: true; Layout.topMargin: 14; Layout.bottomMargin: 16
                        implicitHeight: naCol.implicitHeight + 20
                        radius: 6; color: root.t.well; border.color: root.t.bd
                        Column {
                            id: naCol
                            x: 12; y: 10; width: parent.width - 24
                            spacing: 2
                            Label { visible: detail.finished; width: naCol.width; wrapMode: Text.Wrap; text: "A comment does not bring it back to the board."; font.pointSize: root.smallSize; font.weight: Font.DemiBold }
                            Label {
                                visible: detail.finished; width: naCol.width; wrapMode: Text.Wrap; bottomPadding: na.lines.length > 0 ? 6 : 0
                                text: "It is added to the record, and the issue stays where it is. To bring it back, reopen it."; font.pointSize: root.smallSize; color: root.t.muted
                            }
                            Label { visible: na.lines.length > 0; text: "Not available to you:"; font.pointSize: root.smallSize; font.weight: Font.DemiBold }
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
                    Rectangle { Layout.fillWidth: true; Layout.bottomMargin: 14; implicitHeight: 1; color: root.t.bd }
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
                }
            }
        }
    }
}
