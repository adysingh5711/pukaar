// Pukaar view. QML-only: every call goes to pukaar_core through the host's
// `logos` bridge, and the view re-reads state every 2 s (no module events at L1).
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    id: root
    property var me: ({})
    readonly property var noInfo: ({ categories: [], locations: [], members: [], pending: [] })
    property var info: noInfo
    // The lists of site_info live in properties of their own, written only when their content
    // changed: info itself changes on every received event (it carries an event count), and a
    // rebuilt model would wipe typed text in the Members rows and reset the Report combos.
    property var categories: []
    property var locations: []
    property var members: []
    property var pending: []
    readonly property var groupNames: groups()
    property var issues: []
    property var selected: null          // issue_timeline() result
    property var checkpoint: null        // checkpoint_now() result
    property string message: ""
    property bool onlyMine: false
    property bool busy: false            // a call is running: every action button waits (no double submit)
    readonly property bool inSite: !!(me && me.site)
    readonly property bool staff: me.role === "Steward" || me.role === "Admin"
    readonly property bool isAdmin: me.role === "Admin"
    // A restored identity waits for its own history before signing anything (core refuses meanwhile).
    readonly property bool syncing: !!me.syncing_own_history
    readonly property string otherLabel: "Other (not on the list)"

    // ---- design tokens: the only place a colour or the stage->colour map is spelled out ----
    readonly property color dangerColor: "#d73a49"
    readonly property color mutedColor: "gray"
    readonly property var stageColors: ({
        Open: "#b45309",
        Acknowledged: "#b45309",
        InProgress: "#1d6fd8",
        AwaitingConfirmation: "#7c3aed",
        ConfirmedResolved: "#15803d",
        ClosedWontfix: "#57606a",
        Duplicate: "#57606a"
    })
    function stageColor(status) { return root.stageColors[status] || root.dangerColor }
    // Statuses a steward can still act on (acknowledge / update / claim / won't-fix).
    function isActionable(status) { return ["Open", "Acknowledged", "InProgress"].indexOf(status) >= 0 }

    // Raw reply of each polled method as last applied. Not bound to anything: poll() compares
    // against it and skips the assignment when nothing changed, so a quiet 2 s tick resets no model.
    property var seen: ({})

    // This is the ONE place that talks to pukaar_core: every button routes through run()/call()/poll().
    function callRaw(method, args) {
        // qmllint disable unqualified
        // `logos` is injected by the C++ host at runtime; it has no static QML type.
        if (typeof logos === "undefined" || !logos.callModule)
            return "error: no logos bridge (run inside Basecamp or logos-standalone-app)"
        return logos.callModule("pukaar_core", method, args)
        // qmllint enable unqualified
    }
    // Replies arrive as JSON strings, sometimes double-encoded by the bridge.
    function decode(v) {
        try { v = JSON.parse(v) } catch (e) {}
        if (typeof v === "string") { try { v = JSON.parse(v) } catch (e) {} }
        return v
    }
    function call(method, args) { return decode(callRaw(method, args)) }
    // Read-only refresh: hand the decoded reply to `apply` only if it differs from the last one.
    function poll(method, args, apply) {
        var raw = callRaw(method, args)
        if (raw === seen[method]) return
        seen[method] = raw
        apply(decode(raw))
    }
    function keep(name, value) { if (JSON.stringify(value) !== JSON.stringify(root[name])) root[name] = value }
    function setInfo(i) {
        info = i
        keep("categories", i.categories); keep("locations", i.locations)
        keep("members", i.members); keep("pending", i.pending)
    }
    function openIssue(id) { poll("issue_timeline", [id], function (t) { if (t && t.issue) selected = t }) }
    function closeIssue() { selected = null; delete seen.issue_timeline }
    function failed(v) {
        return typeof v === "string" && (v.indexOf("error:") === 0 || v.indexOf("rejected:") === 0)
    }
    // Every action goes through here. `onOk(reply)` runs only when the core accepted it, so
    // typed text survives a refusal and the user can fix it and retry.
    function run(method, args, onOk) {
        busy = true
        var r = call(method, args)
        busy = false
        message = failed(r) ? r : ""
        refresh()
        if (!failed(r) && onOk) onOk(r)
    }
    // Drop this device's copy of the site (the key stays); back to the first-run screen.
    function leave() {
        run("leave_site", [], function () {
            closeIssue(); checkpoint = null; issues = []; setInfo(noInfo); seen = ({})
        })
    }
    function refresh() {
        poll("my_identity", [], function (m) { me = (m && typeof m === "object") ? m : {} })
        if (!inSite) return
        poll("site_info", [], function (i) { if (i && i.site) setInfo(i) })
        poll("list_issues", [], function (l) { if (Array.isArray(l)) issues = l })
        if (selected) poll("issue_timeline", [selected.issue.id], function (t) { if (t && t.issue) selected = t })
    }
    function inColumn(statuses) {
        return issues.filter(function (i) {
            return statuses.indexOf(i.status) >= 0 && (!onlyMine || i.reporter === me.key)
        })
    }
    function who(key, name) { return name ? name : "pseudonym " + String(key).substr(0, 6) }
    function when(ts) { return new Date(ts * 1000).toLocaleString(Qt.locale(), "d MMM HH:mm") }
    function place(i) {
        var p = i.location === "other" ? "Other" : (i.location + " " + (i.location_label || ""))
        return i.landmark ? p + " (" + i.landmark + ")" : p
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
    function groups() {
        var g = []
        locations.forEach(function (l) { if (g.indexOf(l.group) < 0) g.push(l.group) })
        g.push(otherLabel)
        return g
    }
    function locationsIn(group) { return locations.filter(function (l) { return l.group === group }) }

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
            elide: Text.ElideRight; text: chip.label; color: "white"; font.pixelSize: 11
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
        required property color tint
        required property color alertColor
        required property color mutedColor
        signal opened()
        width: ListView.view.width
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
                font.pixelSize: 11
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
        color: row.rejected ? row.alertColor : palette.text
        text: row.lineText
    }

    // One tab of the StackLayout: scrolls when its content outgrows the window (30 pending
    // members, a long checkpoint). Content is laid out as a column of the page's width.
    component TabPage: ScrollView {
        id: page
        default property alias content: column.data
        contentWidth: availableWidth
        clip: true
        ColumnLayout { id: column; width: page.availableWidth; spacing: 8 }
    }

    // A button that also waits while a call is running. `allowed` is the caller's own
    // precondition; submit() is what Return in a neighbouring field calls (it honours both).
    component ActionButton: Button {
        property bool allowed: true
        enabled: allowed && !root.busy
        function submit() { if (enabled) clicked() }
    }

    // A text field with its submit button glued on — the "paste id / type name, then act" row.
    component LabelledField: RowLayout {
        id: lf
        property alias text: field.text
        property alias placeholder: field.placeholderText
        property alias echoMode: field.echoMode
        property string buttonText: "Go"
        property bool buttonEnabled: true
        signal submitted()
        TextField { id: field; Layout.fillWidth: true; onAccepted: go.submit() }
        ActionButton { id: go; text: lf.buttonText; allowed: lf.buttonEnabled; onClicked: lf.submitted() }
    }

    Timer { interval: 2000; running: true; repeat: true; triggeredOnStart: true; onTriggered: root.refresh() }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        RowLayout {
            Label { text: "Pukaar"; font.pixelSize: 20; font.bold: true }
            Label { text: root.inSite ? (root.info.name || "") : "not in a site yet"; opacity: 0.7 }
            Item { Layout.fillWidth: true }
            TextEdit {   // selectable, so the admin can copy the site id to share
                visible: root.inSite; readOnly: true; selectByMouse: true
                text: "site " + (root.me.site || ""); font.pixelSize: 10; color: root.mutedColor
            }
            Label {   // Delivery bring-up status from pukaar_core; errors stay visible until it recovers
                text: "delivery " + (root.me.delivery || "…")
                color: root.failed(root.me.delivery) ? root.dangerColor : root.mutedColor
            }
            Label {
                visible: root.inSite
                text: "you: " + (root.me.name || "pseudonym") + " · " + (root.me.fingerprint || "") + " · "
                      + (root.me.role || "pending: read your fingerprint at the kiosk")
            }
        }
        Label { visible: root.message !== ""; text: root.message; color: root.dangerColor; wrapMode: Text.Wrap; Layout.fillWidth: true }
        Label {   // forks: someone signed two versions of their log (the threat model says: flag it)
            visible: root.inSite && root.info.forks > 0
            Layout.fillWidth: true; wrapMode: Text.Wrap; color: root.dangerColor; font.bold: true
            text: "Integrity warning: " + root.info.forks + " conflicting history pair(s) found. Someone signed two versions of their log, so devices may show different boards."
        }
        RowLayout {   // joined, but the genesis hasn't arrived: wrong id, or nobody online yet
            visible: root.inSite && !root.info.name
            Layout.fillWidth: true
            Label {
                Layout.fillWidth: true; wrapMode: Text.Wrap
                text: "Waiting for site data… If this lasts, check the site id, or wait for a member to come online."
            }
            ActionButton { text: "Leave this site"; onClicked: root.leave() }
        }
        RowLayout {
            visible: root.inSite && root.syncing
            Layout.fillWidth: true
            Label {
                Layout.fillWidth: true; wrapMode: Text.Wrap; color: root.dangerColor
                text: "Restoring your earlier reports from the network. Actions are paused until they arrive, so your new ones can't conflict with them."
            }
            ActionButton { text: "Skip waiting (history lost)"; onClicked: root.run("skip_history_sync", []) }
        }

        // ---------- first run: create or join ----------
        ColumnLayout {
            visible: !root.inSite
            Layout.fillWidth: true
            Label { text: "Join a site (paste the site id shown on the kiosk)" }
            LabelledField {
                id: joinField
                placeholder: "64 hex characters"
                buttonText: "Join"
                onSubmitted: root.run("site_join", [joinField.text])
            }
            Label { text: "Or create one (site admin only). Edit the location list, then give your name." }
            TextArea {
                id: genesis
                Layout.fillWidth: true
                Layout.preferredHeight: 160
                wrapMode: TextEdit.Wrap
                text: '{"Genesis":{"name":"Dhun",'
                    + '"categories":["water","waste","power","access","rooms","kitchen","safety"],'
                    + '"locations":['
                    + '{"code":"W-01","label":"Tap, dining hall","group":"Water points"},'
                    + '{"code":"W-02","label":"Tap, tent row A","group":"Water points"},'
                    + '{"code":"B-01","label":"Bin, main path","group":"Bins"},'
                    + '{"code":"P-01","label":"Gate to the farm path","group":"Paths"},'
                    + '{"code":"R-01","label":"Tent 1","group":"Rooms and tents"}],'
                    + '"sla_ack_h":12,"sla_fix_h":48,"max_open_per_author":10}}'
            }
            LabelledField {   // staff are always named: the name goes into the genesis, not the JSON above
                id: adminField
                placeholder: "Your name, as residents will see it"
                buttonText: "Create site"
                buttonEnabled: adminField.text.trim() !== ""
                onSubmitted: {
                    var g
                    try {
                        g = JSON.parse(genesis.text)
                        g.Genesis.admin_name = adminField.text.trim()
                    } catch (e) {
                        root.message = "error: site settings: " + e
                        return
                    }
                    root.run("site_create", [JSON.stringify(g)])
                }
            }
        }

        // ---------- first run: restore a backed-up identity ----------
        ColumnLayout {
            visible: !root.inSite
            Layout.fillWidth: true
            Label { text: "Or restore your identity from a backup (Identity tab, on your other device or before you reinstalled)" }
            TextArea {
                id: importBlob
                Layout.fillWidth: true
                Layout.preferredHeight: 70
                wrapMode: TextEdit.WrapAnywhere
                placeholderText: "pukaar-id-1:…"
            }
            LabelledField {
                id: importPassword
                placeholder: "backup password"
                echoMode: TextInput.Password
                buttonText: "Import identity"
                buttonEnabled: importBlob.text.trim() !== "" && importPassword.text !== ""
                onSubmitted: {
                    root.run("import_identity", [importBlob.text, importPassword.text],
                             function () { importBlob.text = ""; importPassword.text = "" })
                }
            }
        }

        // ---------- in a site ----------
        TabBar {
            id: tabs
            visible: root.inSite
            Layout.fillWidth: true
            TabButton { text: "Board" }
            TabButton { text: "Report" }
            TabButton { text: "Members"; enabled: root.isAdmin }
            TabButton { text: "Anchor" }
            TabButton { text: "Identity" }
        }

        StackLayout {
            visible: root.inSite
            currentIndex: tabs.currentIndex
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Board (the brief's three stages) + timeline
            ColumnLayout {
                CheckBox { text: "Only my reports"; checked: root.onlyMine; onToggled: root.onlyMine = checked }
                RowLayout {
                    spacing: 8
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
                            Layout.fillHeight: true
                            Layout.preferredWidth: 200
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
                                    tint: root.stageColor(modelData.status)
                                    alertColor: root.dangerColor
                                    mutedColor: root.mutedColor
                                    onOpened: root.openIssue(modelData.id)
                                }
                            }
                        }
                    }

                    // Timeline: every step anyone took, and the actions allowed from here
                    ColumnLayout {
                        id: detail
                        visible: root.selected !== null
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        property var issue: root.selected ? root.selected.issue : ({})
                        property string st: issue.status || ""
                        property bool mine: issue.reporter === root.me.key
                        property bool claimant: issue.claimant === root.me.key
                        property bool hasNote: note.text.trim() !== ""   // the core rejects these without one
                        function act(a, eta) {
                            root.run("act", [issue.id, a, note.text, nextStep.text, eta | 0],
                                     function () { note.text = ""; nextStep.text = "" })
                        }
                        Label { text: detail.issue.category + " · " + root.place(detail.issue) + " · " + detail.issue.stage; font.bold: true; wrapMode: Text.Wrap; Layout.fillWidth: true }
                        Label { text: detail.issue.text; wrapMode: Text.Wrap; Layout.fillWidth: true }
                        ListView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true
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
                        TextField { id: note; Layout.fillWidth: true; maximumLength: 500; placeholderText: "note: what was seen / done / why"; onAccepted: comment.submit() }
                        RowLayout {
                            visible: root.staff && root.isActionable(detail.st)
                            TextField { id: nextStep; Layout.fillWidth: true; maximumLength: 500; placeholderText: "next step (for updates)" }
                            Label { text: "ETA h" }
                            SpinBox { id: eta; from: 0; to: 168; value: 4 }
                        }
                        Flow {
                            Layout.fillWidth: true
                            enabled: !root.syncing
                            spacing: 6
                            ActionButton { visible: root.staff && detail.st === "Open"; text: "Acknowledge"; onClicked: detail.act("acknowledge", eta.value) }
                            ActionButton { visible: root.staff && root.isActionable(detail.st); allowed: detail.hasNote; text: "Post update"; onClicked: detail.act("update", eta.value) }
                            ActionButton { visible: root.staff && root.isActionable(detail.st); allowed: detail.hasNote; text: "Claim fixed (say what was done)"; onClicked: detail.act("claim_resolved", 0) }
                            ActionButton { visible: root.staff && root.isActionable(detail.st); allowed: detail.hasNote; text: "Won't fix (reason)"; onClicked: detail.act("close_wontfix", 0) }
                            ActionButton {
                                visible: detail.st === "AwaitingConfirmation" && !detail.claimant && (detail.mine || root.me.role === "Resident")
                                text: detail.mine ? "Confirm it's fixed" : "Confirm (1 of 2 residents)"
                                onClicked: detail.act("confirm", 0)
                            }
                            ActionButton {
                                visible: ["AwaitingConfirmation", "ConfirmedResolved", "ClosedWontfix", "Duplicate"].indexOf(detail.st) >= 0 && (detail.mine || root.me.role === "Resident")
                                allowed: detail.hasNote
                                text: "Reopen (reason)"
                                onClicked: detail.act("reopen", 0)
                            }
                            ActionButton { id: comment; allowed: detail.hasNote; text: "Comment"; onClicked: detail.act("comment", 0) }
                            Row {   // staff: close this one as a duplicate of another open issue
                                visible: root.staff && root.isActionable(detail.st)
                                spacing: 6
                                ComboBox {
                                    // The model is rebuilt on every 2 s refresh, which resets currentIndex,
                                    // so the user's pick is kept in `target`, never read from currentIndex.
                                    id: dupOf
                                    property string target: ""
                                    property string targetLabel: ""
                                    property string forIssue: detail.issue.id || ""
                                    onForIssueChanged: target = ""
                                    width: 260
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
                            Button { text: "Close"; flat: true; onClicked: root.closeIssue() }
                        }
                    }
                }
            }

            // Report: pick a group, then a place; or "Other" plus a landmark
            TabPage {
                enabled: !root.syncing
                Label { text: "What's wrong?" }
                ComboBox { id: category; model: root.categories; Layout.preferredWidth: 240 }
                RowLayout {
                    ComboBox { id: group; model: root.groupNames; Layout.preferredWidth: 220 }
                    ComboBox {
                        id: location
                        visible: group.currentText !== root.otherLabel
                        Layout.preferredWidth: 320
                        model: root.locationsIn(group.currentText).map(function (l) { return l.code + " · " + l.label })
                    }
                }
                TextField {
                    id: landmark
                    Layout.fillWidth: true
                    maximumLength: 500
                    placeholderText: group.currentText === root.otherLabel
                        ? "Required: describe the place (e.g. pipe behind tent 4, by the neem tree)"
                        : "Optional landmark (e.g. the left tap, behind the water tank)"
                }
                // maximumLength counts characters; the core's 500 limit is bytes (Hindi is 3 B/char),
                // so a long non-Latin line comes back as "error: too long" instead.
                TextField {
                    id: reportText; Layout.fillWidth: true; maximumLength: 500
                    placeholderText: "One line: what's wrong"; onAccepted: reportBtn.submit()
                }
                ActionButton {
                    id: reportBtn
                    text: "Report"
                    allowed: reportText.text.trim().length > 0
                             && (group.currentText === root.otherLabel ? landmark.text.trim().length > 0 : location.currentIndex >= 0)
                    onClicked: {
                        var code = group.currentText === root.otherLabel
                            ? "other" : root.locationsIn(group.currentText)[location.currentIndex].code
                        root.run("report", [category.currentText, code, landmark.text, reportText.text],
                                 function () { reportText.text = ""; landmark.text = ""; tabs.currentIndex = 0 })
                    }
                }
                Label {
                    visible: !root.staff
                    text: "Your name (leave empty to stay a pseudonym; the admin who granted your role can still link it). Staff are always named."
                    wrapMode: Text.Wrap; Layout.fillWidth: true
                }
                LabelledField {
                    id: nameField
                    visible: !root.staff
                    placeholder: "display name"
                    buttonText: "Set name"
                    onSubmitted: root.run("set_profile", [nameField.text])
                }
            }

            // Members (admin): grant pending keys after the fingerprint is read aloud
            TabPage {
                enabled: !root.syncing
                Label { text: "Pending: grant only after the person reads this fingerprint aloud"; font.bold: true }
                Repeater {
                    model: root.pending
                    delegate: Flow {
                        id: pendingRow
                        Layout.fillWidth: true
                        spacing: 6
                        required property var modelData
                        Label { text: pendingRow.modelData.fingerprint + "  " + (pendingRow.modelData.name || "(no name)"); font.family: "monospace" }
                        ActionButton { text: "Grant resident"; onClicked: root.run("grant_role", [pendingRow.modelData.key, "resident", ""]) }
                        TextField { id: staffName; placeholderText: "steward's real name"; Layout.preferredWidth: 160; onAccepted: grantSteward.submit() }
                        ActionButton {
                            id: grantSteward
                            text: "Grant steward"
                            allowed: staffName.text.trim().length > 0
                            onClicked: root.run("grant_role", [pendingRow.modelData.key, "steward", staffName.text])
                        }
                    }
                }
                Label { text: "Members"; font.bold: true }
                Repeater {
                    model: root.members
                    delegate: Flow {
                        id: memberRow
                        Layout.fillWidth: true
                        spacing: 6
                        required property var modelData
                        Label {
                            text: memberRow.modelData.fingerprint + "  " + memberRow.modelData.role + "  " + (memberRow.modelData.name || "pseudonym")
                            font.family: "monospace"
                        }
                        TextField {
                            id: revokeReason; visible: memberRow.modelData.key !== root.me.key; maximumLength: 500
                            placeholderText: "reason for revoking"; Layout.preferredWidth: 200; onAccepted: revoke.submit()
                        }
                        ActionButton {
                            id: revoke
                            visible: revokeReason.visible
                            text: "Revoke"
                            allowed: revokeReason.text.trim().length > 0
                            onClicked: root.run("revoke_role", [memberRow.modelData.key, revokeReason.text])
                        }
                    }
                }
                Label { text: "Add a location (for example, found on the site walk)"; font.bold: true }
                RowLayout {
                    ComboBox { id: locGroup; editable: true; model: root.groupNames.slice(0, -1); Layout.preferredWidth: 180 }
                    TextField { id: locCode; placeholderText: "W-04"; Layout.preferredWidth: 80 }
                    TextField { id: locLabel; placeholderText: "Tap behind tent 4"; Layout.fillWidth: true; onAccepted: addLoc.submit() }
                    ActionButton {
                        id: addLoc
                        text: "Add"
                        allowed: locCode.text.trim() !== "" && locLabel.text.trim() !== "" && locGroup.editText.trim() !== ""
                        onClicked: root.run("add_location", [locCode.text, locLabel.text, locGroup.editText],
                                            function () { locCode.text = ""; locLabel.text = "" })
                    }
                }
            }

            // Anchor: compute, run the printed spel command in a terminal, record the reference
            TabPage {
                enabled: !root.syncing
                ActionButton { text: "Compute checkpoint"; onClicked: root.checkpoint = root.call("checkpoint_now", []) }
                Label { visible: !!root.checkpoint; text: root.checkpoint ? root.checkpoint.n_events + " events, root " + root.checkpoint.heads_root.substr(0, 16) + "…" : "" }
                TextArea {
                    visible: !!root.checkpoint
                    readOnly: true; selectByMouse: true; wrapMode: TextEdit.WrapAnywhere
                    Layout.fillWidth: true
                    text: root.checkpoint ? root.checkpoint.spel : ""
                }
                LabelledField {
                    id: anchorField
                    visible: !!root.checkpoint
                    placeholder: "tx hash or pda:<account id> printed by spel"
                    buttonText: "Record anchor"
                    onSubmitted: {
                        root.run("record_anchor", [JSON.stringify(root.checkpoint.heads), anchorField.text],
                                 function () { anchorField.text = ""; root.checkpoint = null })
                    }
                }
            }

            // Identity: a password-sealed backup, to continue as the same person after a reinstall
            TabPage {
                Label { text: "Back up your identity"; font.bold: true }
                Label {
                    text: "Keep this and your password safe. Anyone with both can act as you. Never run the same identity on two devices at once."
                    color: root.dangerColor; wrapMode: Text.Wrap; Layout.fillWidth: true
                }
                TextField {
                    id: exportPassword
                    Layout.fillWidth: true
                    echoMode: TextInput.Password
                    placeholderText: "password (at least 8 characters)"
                }
                LabelledField {
                    id: exportConfirm
                    placeholder: "same password again"
                    echoMode: TextInput.Password
                    buttonText: "Export identity"
                    buttonEnabled: exportPassword.text.length >= 8 && exportConfirm.text === exportPassword.text
                    onSubmitted: {
                        root.run("export_identity", [exportPassword.text],
                                 function (r) { exportOut.text = r; exportPassword.text = ""; exportConfirm.text = "" })
                    }
                }
                TextArea {
                    id: exportOut
                    visible: text !== ""
                    readOnly: true; selectByMouse: true; wrapMode: TextEdit.WrapAnywhere
                    Layout.fillWidth: true
                }
                Button { visible: exportOut.text !== ""; text: "Hide"; flat: true; onClicked: exportOut.text = "" }
            }
        }
    }
}
