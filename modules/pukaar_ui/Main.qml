// Pukaar view. QML-only: every call goes to pukaar_core through the host's
// `logos` bridge, and the view re-reads state every 2 s (no module events at L1).
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts

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
    // One light theme. A dark theme means swapping the values of this block, nothing else.
    readonly property color surfaceColor: "#ffffff"       // page, fields, list rows
    readonly property color surfaceAltColor: "#f3f4f6"    // buttons, tabs, cards, group headers
    readonly property color hoverColor: "#e8eaed"         // pointer over a row or a menu item
    readonly property color textColor: "#1f2328"          // 15:1 on surface
    readonly property color mutedColor: "#57606a"         // 6:1 on white (plain "gray" was 3.95:1)
    readonly property color borderColor: "#8c959f"        // 3:1 on white: the box edge is a UI boundary
    readonly property color accentColor: "#1d6fd8"        // focus ring, selected tab, selection
    readonly property color accentSoftColor: "#dbe8f9"    // the picked row
    readonly property color accentTextColor: "#ffffff"    // text on accent or on a stage chip
    readonly property color dangerColor: "#d73a49"
    // Every control reads these roles. Set once on the root, so nothing falls back to the
    // system palette (a dark OS would otherwise give light-on-light or dark-on-dark controls).
    // Same value in the disabled and inactive groups: disabled controls fade with `opacity`.
    palette {
        window: root.surfaceColor; windowText: root.textColor
        base: root.surfaceColor; alternateBase: root.surfaceAltColor; text: root.textColor
        button: root.surfaceAltColor; buttonText: root.textColor; brightText: root.accentTextColor
        placeholderText: root.mutedColor
        highlight: root.accentColor; highlightedText: root.accentTextColor
        link: root.accentColor; linkVisited: root.accentColor
        toolTipBase: root.surfaceColor; toolTipText: root.textColor
        light: root.surfaceColor; midlight: root.hoverColor; mid: root.borderColor
        dark: root.mutedColor; shadow: root.textColor
    }
    // Sizes follow the user's system font instead of fixed pixels.
    FontMetrics { id: systemFont }       // default font = the application's
    readonly property real baseSize: systemFont.font.pointSize > 0 ? systemFont.font.pointSize : 10
    readonly property real smallSize: baseSize * 0.85
    readonly property real titleSize: baseSize * 1.6
    readonly property var stageColors: ({
        Open: "#b45309",
        Acknowledged: "#b45309",
        InProgress: accentColor,
        AwaitingConfirmation: "#7c3aed",
        ConfirmedResolved: "#15803d",
        ClosedWontfix: mutedColor,
        Duplicate: mutedColor
    })
    function stageColor(status) { return root.stageColors[status] || root.dangerColor }
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
    function place(i) {
        var p = i.location === "other" ? "Other" : (i.location + " " + (i.location_label || ""))
        if (i.location_retired) p += " (retired)"
        return i.landmark ? p + " (" + i.landmark + ")" : p
    }
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
    // Overdue flags are computed by the core (site SLA hours); this only words them.
    function slaLine(i) {
        if (i.ack_overdue) return "Not acknowledged within " + info.sla_ack_h + " h. "
        if (i.fix_overdue) return "Past the " + info.sla_fix_h + " h fix target. "
        if (i.awaiting_48h) return "Fix unconfirmed for over 48 h. "
        return ""
    }
    function flagged(i) { return isOverdue(i) || !!i.ack_overdue || !!i.fix_overdue || !!i.awaiting_48h }
    // The card's grey line: age, whose move it is, SLA flags, reopen count, next step.
    function hint(i) {
        var waiting = i.status !== "AwaitingConfirmation" ? ""
            : i.reporter === me.key ? "Needs your confirmation. " : "Fix claimed: waiting for the reporter to confirm. "
        return "Reported " + age(i.reported_ts) + ". " + waiting + slaLine(i)
            + (i.reopen_count > 0 ? "Reopened " + i.reopen_count + "×. " : "") + nextLine(i)
    }
    // "Next: fit washer · due in 5 h" / "· OVERDUE by 2 h"
    function nextLine(i) {
        var p = i.progress
        if (!p) return i.status === "Open" ? "Waiting for a steward to acknowledge" : ""
        var t = p.next_step ? "Next: " + p.next_step : p.note
        if (p.due_ts > 0 && isActionable(i.status)) {
            var h = dueHours(p)
            t += h >= 0 ? " · due in " + h + " h" : " · OVERDUE by " + (-h) + " h"
        }
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

    // ---- controls: one frame (border token, blue when focused) shared by every control, so no
    // field, button, combo or box is ever borderless whatever style the host uses ----
    component Frame: Rectangle {
        property bool ring: false          // keyboard/typing focus: thicker, blue
        property color fill: root.surfaceColor
        color: fill
        radius: 4
        border.width: ring ? 2 : 1
        border.color: ring ? root.accentColor : root.borderColor
    }
    component FramedField: TextField {
        id: field
        selectByMouse: true
        leftPadding: 8; rightPadding: 8
        opacity: enabled ? 1 : 0.5
        background: Frame { ring: field.activeFocus }
    }
    component FramedButton: Button {
        id: fb
        leftPadding: 12; rightPadding: 12; topPadding: 6; bottomPadding: 6
        opacity: enabled ? 1 : 0.5
        background: Frame { ring: fb.visualFocus; fill: fb.down ? Qt.darker(root.surfaceAltColor, 1.12) : root.surfaceAltColor }
    }
    component FramedCombo: ComboBox {
        id: combo
        leftPadding: 10; rightPadding: 28; topPadding: 6; bottomPadding: 6
        opacity: enabled ? 1 : 0.5
        background: Frame { ring: combo.visualFocus }
        contentItem: Label { text: combo.displayText; elide: Text.ElideRight; verticalAlignment: Text.AlignVCenter }
        indicator: Label { x: combo.width - width - 10; y: (combo.height - height) / 2; text: "\u25be" }
        // The style's own row paints its highlight from palette roles that are not meant for it
        // (invisible text on some palettes); this one reads the tokens.
        delegate: ItemDelegate {
            id: option
            required property var model
            required property int index
            width: ListView.view.width
            text: model[combo.textRole]
            highlighted: combo.highlightedIndex === index
            background: Rectangle { color: option.highlighted ? root.hoverColor : root.surfaceColor }
            contentItem: Label { text: option.text; verticalAlignment: Text.AlignVCenter; font.bold: combo.currentIndex === option.index }
        }
    }
    component FramedCheck: CheckBox {
        id: check
        opacity: enabled ? 1 : 0.5
        indicator: Frame {
            ring: check.visualFocus
            x: check.leftPadding; y: (check.height - height) / 2; width: 18; height: 18
            Label { anchors.centerIn: parent; text: "\u2713"; visible: check.checked }
        }
        contentItem: Label { text: check.text; leftPadding: check.indicator.width + 8; verticalAlignment: Text.AlignVCenter }
    }
    component PageTab: TabButton {
        id: tab
        background: Frame {
            fill: tab.checked ? root.surfaceColor : root.surfaceAltColor
            Rectangle { visible: tab.checked; width: parent.width; height: 3; anchors.bottom: parent.bottom; color: root.accentColor }
        }
        contentItem: Label {
            text: tab.text; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter
            font.bold: tab.checked
        }
        opacity: enabled ? 1 : 0.5
    }

    // ---- reusable pieces (props in, signals out; no reach into the enclosing scope) ----

    // Small coloured pill for an issue's stage. Colour always comes from stageColor() above.
    component StatusChip: Rectangle {
        id: chip
        required property string label
        required property color tint
        implicitWidth: chipText.implicitWidth + 12
        implicitHeight: chipText.implicitHeight + 4
        radius: height / 2
        color: chip.tint
        Label {   // elides when the parent narrows the chip below its natural width
            id: chipText; anchors.centerIn: parent; width: Math.min(implicitWidth, chip.width - 12)
            elide: Text.ElideRight; text: chip.label; color: root.accentTextColor; font.pointSize: root.smallSize
        }
    }

    // One board card. All text is pre-computed by the caller (place/hint/stageColor),
    // so this stays a pure presentational piece reusable across all four columns.
    component IssueCard: ItemDelegate {
        id: card
        required property var issue
        required property string placeText
        required property string hintText
        required property bool overdue
        required property bool current       // the issue open in the timeline pane
        required property color tint
        required property color alertColor
        required property color mutedColor
        signal opened()
        width: ListView.view.width
        padding: 8
        background: Frame { ring: card.visualFocus || card.current; fill: card.down ? Qt.darker(root.surfaceAltColor, 1.06) : root.surfaceAltColor }
        Accessible.name: card.placeText + ": " + card.issue.text
        Accessible.description: card.issue.stage + ". " + card.hintText
        contentItem: Column {
            width: card.width
            spacing: 2
            Label { width: parent.width; wrapMode: Text.Wrap; font.bold: true; text: card.placeText }
            Label { width: parent.width; wrapMode: Text.Wrap; text: card.issue.text }
            // Stacked, not side by side: a wide chip must never squeeze the hint to a sliver.
            StatusChip { width: Math.min(implicitWidth, parent.width); label: card.issue.stage; tint: card.tint }
            Label {
                width: parent.width
                wrapMode: Text.Wrap
                font.pointSize: root.smallSize
                color: card.overdue ? card.alertColor : card.mutedColor
                text: card.hintText
            }
        }
        onClicked: card.opened()
    }

    // One timeline entry. The caller composes the full line (who/when/kind/body/rejected/anchored).
    component TimelineRow: Label {
        id: row
        required property string lineText
        required property bool rejected
        required property color alertColor
        width: ListView.view.width
        wrapMode: Text.Wrap
        color: row.rejected ? row.alertColor : root.textColor
        text: row.lineText
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
            background: Frame { ring: header.visualFocus; fill: header.hovered ? root.hoverColor : root.surfaceAltColor }
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
        Note { text: rule.hint; color: root.mutedColor; font.pointSize: root.smallSize }
    }

    // A visible label above whatever is put inside (every field has one, not only a placeholder).
    component FormRow: ColumnLayout {
        id: form
        property string label
        Layout.fillWidth: true
        spacing: 4
        Label { visible: form.label !== ""; text: form.label; font.pointSize: root.smallSize }
    }

    // A full-width notice: wrapped text, and optionally one button on the right.
    component Banner: RowLayout {
        id: banner
        property string text
        property color tint: root.textColor
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
        opacity: shown ? (enabled ? 1 : 0.5) : 0
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
        property color captionColor: root.textColor
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
        color: root.surfaceAltColor
        Label { id: title; x: 8; anchors.verticalCenter: parent.verticalCenter; text: parent.section; font.bold: true; color: root.mutedColor }
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
        background: Rectangle { color: choice.picked ? root.accentSoftColor : choice.hovered ? root.hoverColor : root.surfaceColor }
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
            color: root.mutedColor
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

    Rectangle { anchors.fill: parent; color: root.surfaceColor }     // the page never shows the host's window colour

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            spacing: 12
            Label { text: "Pukaar"; font.pointSize: root.titleSize; font.bold: true }
            Label { Layout.fillWidth: true; elide: Text.ElideRight; text: root.inSite ? (root.info.name || "") : "not in a site yet"; color: root.mutedColor }
            Label {   // Delivery bring-up status from pukaar_core; errors stay visible until it recovers
                Layout.maximumWidth: root.width / 2
                elide: Text.ElideRight
                text: "delivery: " + root.deliveryLabel(root.me.delivery)
                color: root.failed(root.me.delivery) ? root.dangerColor : root.mutedColor
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
            Label { text: "site " + root.shortId(root.me.site || ""); font.pointSize: root.smallSize; color: root.mutedColor }
            FramedButton {
                text: "Copy"
                Accessible.name: "Copy the full site id"
                onClicked: root.copyText(root.me.site)
            }
        }
        Banner {   // the last refused action; stays until dismissed or the next action replaces it
            visible: root.message !== ""
            text: root.message; tint: root.dangerColor
            buttonText: "×"; buttonName: "Dismiss this error"
            onActivated: root.message = ""
        }
        Banner {   // forks: someone signed two versions of their log (the threat model says: flag it)
            visible: root.inSite && root.info.forks > 0
            tint: root.dangerColor; bold: true
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
            tint: root.dangerColor
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
                placeholder: "64 hex characters"
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
                        FramedField { id: adminName; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Your name" }
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
                        color: root.mutedColor
                        text: "Each place has a group (for the picker), a short code people can say aloud, and a name. Codes can't change later."
                    }
                    RowLayout {   // column titles: the placeholders vanish once a row is filled
                        visible: setup.places.count > 0
                        spacing: 6
                        Label { Layout.preferredWidth: 170; text: "Group"; font.pointSize: root.smallSize; color: root.mutedColor }
                        Label { Layout.preferredWidth: 90; text: "Code"; font.pointSize: root.smallSize; color: root.mutedColor }
                        Label { text: "Name"; font.pointSize: root.smallSize; color: root.mutedColor }
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
                            FramedField { Layout.preferredWidth: 90; maximumLength: 500; text: draftRow.code; placeholderText: "code"; Accessible.name: "Place " + (draftRow.index + 1) + " code"; onTextEdited: setup.edit(draftRow.index, "code", text) }
                            FramedField { Layout.fillWidth: true; maximumLength: 500; text: draftRow.label; placeholderText: "name"; Accessible.name: "Place " + (draftRow.index + 1) + " name"; onTextEdited: setup.edit(draftRow.index, "label", text) }
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
                    Note { color: root.mutedColor; text: "Starts from the form above and replaces it while this section is open. Close it to go back to the form." }
                    FramedTextArea { id: advancedJson; name: "Site settings, JSON"; boxHeight: 160 }
                }
                ActionButton {
                    text: "Create site"
                    allowed: advanced.open ? adminName.text.trim() !== "" : setup.problem === ""
                    onClicked: root.run("site_create", [advanced.open ? setup.withAdmin(advancedJson.text) : setup.genesisJson()], null,
                                        "Start with fewer places, then add the rest after creating, via Members \u2192 Add a location.")
                }
                Note { visible: !advanced.open && setup.problem !== ""; color: root.mutedColor; text: setup.problem }
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
                        placeholder: "pukaar-id-1:…"
                    }
                }
                LabelledField {
                    id: importPassword
                    label: "Backup password"
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
                Note {   // an empty board says why, and what to do
                    visible: root.boardCount === 0
                    Layout.bottomMargin: 6
                    color: root.mutedColor
                    text: root.onlyMine ? "You haven't reported anything yet. Use the Report tab to raise a problem."
                                        : "No reports yet. Use the Report tab to raise the first one."
                }
                RowLayout {
                    spacing: 8
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    RowLayout {   // the four columns; hidden behind the pane on a narrow window
                        visible: !(root.narrow && root.selected)
                        spacing: 8
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Repeater {
                            model: [
                                { title: "Reported", statuses: ["Open", "Acknowledged"] },
                                { title: "In progress", statuses: ["InProgress", "AwaitingConfirmation"] },
                                { title: "Resolved", statuses: ["ConfirmedResolved"] },
                                { title: "Closed without fix", statuses: ["ClosedWontfix", "Duplicate"] }
                            ]
                            delegate: ColumnLayout {
                                id: col
                                required property var modelData
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                Layout.preferredWidth: 1   // equal shares of the width...
                                Layout.minimumWidth: 160   // ...but never thinner than this
                                Label { text: col.modelData.title + " (" + root.inColumn(col.modelData.statuses).length + ")"; font.bold: true }
                                ListView {
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    clip: true
                                    spacing: 4
                                    model: root.inColumn(col.modelData.statuses)
                                    delegate: IssueCard {
                                        required property var modelData
                                        issue: modelData
                                        placeText: root.place(modelData)
                                        hintText: root.hint(modelData)
                                        overdue: root.flagged(modelData)
                                        current: !!root.selected && root.selected.issue.id === modelData.id
                                        tint: root.stageColor(modelData.status)
                                        alertColor: root.dangerColor
                                        mutedColor: root.mutedColor
                                        onOpened: root.openIssue(modelData.id)
                                    }
                                }
                            }
                        }
                    }

                    // Timeline: every step anyone took, and the actions allowed from here
                    ColumnLayout {
                        id: detail
                        visible: root.selected !== null
                        Layout.fillWidth: root.narrow   // wide: a fixed pane beside the board; narrow: it takes over
                        Layout.preferredWidth: root.narrow ? -1 : 340
                        Layout.minimumWidth: 260
                        Layout.fillHeight: true
                        property var issue: root.selected ? root.selected.issue : ({})
                        property string st: issue.status || ""
                        property bool mine: issue.reporter === root.me.key
                        property bool claimant: issue.claimant === root.me.key
                        Keys.onEscapePressed: root.closeIssue()
                        property bool hasNote: note.text.trim() !== ""   // the core rejects these without one
                        readonly property int etaHours: parseInt(eta.text) || 0
                        function act(a, eta) {
                            root.run("act", [issue.id, a, note.text, nextStep.text, eta | 0],
                                     function () { note.text = ""; nextStep.text = "" })
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Label {
                                text: detail.issue.category + " · " + root.place(detail.issue) + " · " + detail.issue.stage
                                    + (detail.issue.location_renamed_from ? " (place renamed from " + detail.issue.location_renamed_from + ")" : "")
                                font.bold: true; wrapMode: Text.Wrap; Layout.fillWidth: true
                            }
                            FramedButton { text: root.narrow ? "← Board" : "Close"; onClicked: root.closeIssue() }
                        }
                        Label { text: detail.issue.text || ""; wrapMode: Text.Wrap; Layout.fillWidth: true }
                        ListView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true
                            spacing: 6
                            model: root.selected ? root.selected.events : []
                            delegate: TimelineRow {
                                required property var modelData
                                alertColor: root.dangerColor
                                rejected: !!modelData.rejected
                                lineText: root.when(modelData.ts) + "  " + modelData.kind + " by " + root.who(modelData.author, modelData.author_name)
                                          + (modelData.body ? ": " + modelData.body : "")
                                          + (modelData.rejected ? "  [rejected: " + modelData.rejected + "]" : "")
                                          + (modelData.anchored_tx ? "  anchor ref " + String(modelData.anchored_tx).substr(0, 16)
                                             + "… recorded by " + root.who(modelData.anchored_by, modelData.anchored_by_name) : "")
                            }
                        }
                        FormRow {
                            label: "Note (what was seen, done or why)"
                            FramedField { id: note; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Note"; onAccepted: comment.submit() }
                        }
                        RowLayout {
                            visible: root.staff && root.isActionable(detail.st)
                            spacing: 8
                            FormRow {
                                label: "Next step (for updates)"
                                FramedField { id: nextStep; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Next step" }
                            }
                            FormRow {
                                label: "ETA (hours)"
                                Layout.fillWidth: false
                                FramedField {
                                    id: eta
                                    Layout.preferredWidth: 80
                                    text: "4"
                                    inputMethodHints: Qt.ImhDigitsOnly
                                    validator: IntValidator { bottom: 0; top: 168 }
                                    Accessible.name: "ETA in hours"
                                }
                            }
                        }
                        Flow {
                            Layout.fillWidth: true
                            enabled: !root.syncing
                            spacing: 6
                            ActionButton { visible: root.staff && detail.st === "Open"; text: "Acknowledge"; onClicked: detail.act("acknowledge", detail.etaHours) }
                            ActionButton { visible: root.staff && root.isActionable(detail.st); allowed: detail.hasNote; text: "Post update"; onClicked: detail.act("update", detail.etaHours) }
                            ActionButton { visible: root.staff && root.isActionable(detail.st); allowed: detail.hasNote; text: "Claim fixed (say what was done)"; onClicked: detail.act("claim_resolved", 0) }
                            ActionButton { visible: root.staff && root.isActionable(detail.st); allowed: detail.hasNote; text: "Won't fix (reason)"; onClicked: detail.act("close_wontfix", 0) }
                            ActionButton {
                                visible: detail.st === "AwaitingConfirmation" && !detail.claimant && (detail.mine || root.me.role === "Resident")
                                text: detail.mine ? "Confirm it's fixed" : "Confirm (" + detail.issue.confirms + " of 2 residents so far)"
                                onClicked: detail.act("confirm", 0)
                            }
                            ActionButton {
                                visible: ["AwaitingConfirmation", "ConfirmedResolved", "ClosedWontfix", "Duplicate"].indexOf(detail.st) >= 0 && (detail.mine || root.me.role === "Resident")
                                allowed: detail.hasNote
                                text: "Reopen (reason)" + (detail.issue.reopen_count > 0 ? ", reopened " + detail.issue.reopen_count + "×" : "")
                                onClicked: detail.act("reopen", 0)
                            }
                            ActionButton { id: comment; allowed: detail.hasNote; text: "Comment"; onClicked: detail.act("comment", 0) }
                            RowLayout {   // staff: close this one as a duplicate of another open issue
                                visible: root.staff && root.isActionable(detail.st)
                                width: parent.width
                                spacing: 6
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
                    color: root.mutedColor
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
                Note { visible: root.approved && reportPage.missing !== ""; color: root.mutedColor; text: reportPage.missing }
                Heading { visible: !root.staff; text: "Your name" }
                Note {
                    visible: !root.staff
                    text: "Leave it empty to stay a pseudonym; the admin who granted your role can still link it. Staff are always named."
                }
                LabelledField {
                    id: nameField
                    visible: !root.staff
                    label: "Display name"
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
                Note { visible: root.pending.length === 0; color: root.mutedColor; text: "No one is waiting." }
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
                            placeholder: "steward's real name"
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
                        placeholder: own ? "you can't revoke yourself" : "reason for revoking"
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
                        FramedField { id: locCode; Layout.preferredWidth: 120; maximumLength: 500; Accessible.name: "Code"; placeholderText: "W-04" }
                    }
                    FormRow {
                        label: "Name"
                        FramedField { id: locLabel; Layout.fillWidth: true; maximumLength: 500; Accessible.name: "Name"; placeholderText: "Tap behind tent 4"; onAccepted: addLoc.submit() }
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
                                    color: locRow.st === "active" ? root.textColor : root.mutedColor
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
                                    placeholderText: locRow.blocked ? "close its issues first" : "reason"
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
                    color: root.mutedColor
                    text: "Removed places are hidden from every list, retired ones included. Removed means hidden: the signed events stay in everyone's log."
                }
                Note {
                    visible: root.removedLocations.length === 0 && membersPage.renamed.length === 0
                    color: root.mutedColor
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
                    placeholder: "tx hash or pda:<account id> printed by spel"
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
                    color: root.dangerColor
                }
                FormRow {
                    label: "Password (at least 8 characters)"
                    FramedField { id: exportPassword; Layout.fillWidth: true; echoMode: TextInput.Password; Accessible.name: "Backup password" }
                }
                LabelledField {
                    id: exportConfirm
                    label: "Repeat the password"
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
