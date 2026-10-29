// Offscreen screenshots of the connection status UI (sidebar dots, Profile "Connection status" card,
// the rejected-write banner) in the states that matter. Run by ui-shots-chain.sh.
// `logos` stands in for the host bridge and answers with small fixtures shaped like pukaar_core's
// my_identity, site_info (chain, storage), chain_status and list_issues. Args after "--": <Main.qml> <out dir>.
import QtQuick

Window {
    id: win
    width: 1280; height: 900; visible: true
    readonly property var args: Qt.application.arguments.slice(Qt.application.arguments.indexOf("--") + 1)

    QtObject {
        id: logos
        property string sc: "refused"
        readonly property int now: Math.floor(Date.now() / 1800000) * 1800
        readonly property string site: "5173".repeat(16)
        readonly property string key: "a1".repeat(32)
        readonly property string badStorage: "storage start was refused: init was refused (see the storage_module log; data dir /m/spike-a/storage) (next try in 20 s)"
        // One state per scenario: the delivery word, the site name, site_info.chain and .storage, chain_status.writes.
        readonly property var scenarios: ({
            refused: { delivery: "Open", name: "",
                chain: { block: 0, ok: false, error: "the site is not on the chain", missing: true, read_ms: 0 },
                storage: { pending: 0, error: badStorage, ready: false },
                writes: [{ id: 1, kind: "create_site", state: "rejected", reason: "storage: storage start was refused", accounts: [] }] },
            healthy: { delivery: "Open", name: "Dhun Camp",
                chain: { block: 48213, ok: true, error: null, missing: false, read_ms: (now - 90) * 1000 },
                storage: { pending: 0, error: null, ready: true },
                writes: [{ id: 7, kind: "set_places", state: "included", reason: null, accounts: [] }] },
            chainerr: { delivery: "error: the delivery module is not running", name: "Dhun Camp",
                chain: { block: 48213, ok: false, error: "request timed out", missing: false, read_ms: (now - 3 * 3600) * 1000 },
                storage: { pending: 3, error: null, ready: true },
                writes: [{ id: 8, kind: "set_role", state: "rejected", reason: "the sequencer did not answer, so the transaction was not sent and nothing was spent", accounts: [] }] },
            funds: { delivery: "Open", name: "Dhun Camp",
                chain: { block: 48213, ok: true, error: null, missing: false, read_ms: (now - 20) * 1000 },
                storage: { pending: 0, error: null, ready: true },
                writes: [{ id: 9, kind: "create_site", state: "rejected", reason: "insufficient balance: the account cannot pay the fee", accounts: [] },
                         { id: 10, kind: "set_places", state: "rejected", reason: "insufficient funds for the fee", accounts: [] }] },
            starting: { delivery: "Starting", name: "Dhun Camp",
                chain: { block: 48213, ok: true, error: null, missing: false, read_ms: (now - 20) * 1000 },
                storage: { pending: 2, error: null, ready: false },
                writes: [] }
        })
        readonly property var s: scenarios[sc]
        function reply(method) {
            switch (method) {
            case "my_identity": return { key: key, fingerprint: "a1a1a1", role: "Admin", name: "Asha Verma", site: site, super_admin: false, delivery: s.delivery }
            case "site_info": return { site: site, name: s.name, categories: ["Water"], locations: [], retired_locations: [], members: [], pending: [],
                                       revoked: [], super_admins: [], chain: s.chain, storage: s.storage, events: 0, forks: 0, sla_ack_h: 24, sla_fix_h: 72 }
            case "chain_status": return { key: "ok", writes: s.writes, waiting: 0, refresh_waiting: 0 }
            case "list_issues": return []
            }
            return "error: the screenshot stub does not do " + method
        }
        function callModule(module, method, a) { return JSON.stringify(reply(method)) }
    }

    Loader { id: view; anchors.fill: parent }
    Item { id: focusSink }

    // scenario, page, width, hover (the tooltip of one dot, shown by hand: the offscreen platform has no mouse)
    readonly property var jobs: [
        ["refused", "board", 1280], ["refused", "profile", 1280], ["refused", "board", 800], ["refused", "profile", 800],
        ["refused", "board", 1280, "Chain"], ["refused", "board", 800, "Delivery"],
        ["healthy", "profile", 1280], ["healthy", "board", 1280, "Storage"], ["chainerr", "profile", 1280], ["chainerr", "profile", 800],
        ["funds", "board", 1280], ["funds", "board", 800], ["starting", "profile", 1280]
    ]
    property int job: -1
    function find(item, name) {
        if (item.objectName === name) return item
        for (var i = 0; i < item.data.length; i++) {   // `data`, not `children`: a ToolTip is a popup, not a child item
            var o = item.data[i], f = o.objectName === name ? o : o.data ? find(o, name) : null
            if (f) return f
        }
        return null
    }
    function next() {
        if (++job >= jobs.length) { Qt.quit(); return }
        var j = jobs[job]
        if (!view.item || logos.sc !== j[0]) {
            logos.sc = j[0]
            view.source = ""
            view.source = "file://" + args[0]
            view.item.now = logos.now
        }
        width = j[2]
        view.item.openPage(j[1])
        focusSink.forceActiveFocus()
        shot.start()
    }
    Timer {
        id: shot; interval: 2500   // the first poll runs at once, then every 2 s: wait until the state is in
        onTriggered: {
            var j = win.jobs[win.job], tip = null
            if (j[3]) {
                tip = win.find(view.item, "statusTip-" + j[3])
                tip.visible = true
                if (!j.armed) { j.armed = true; return shot.restart() }
            }
            win.contentItem.grabToImage(function (r) {
                if (tip) tip.visible = false   // the view is reused by the next job of the same scenario
                r.saveToFile(win.args[1] + "/" + j[0] + "-" + j[1] + "-" + j[2] + (j[3] ? "-hover-" + j[3] : "") + ".png")
                win.next()
            })
        }
    }
    Component.onCompleted: { Qt.application.organization = Qt.application.name = "pukaar-ui-shots"; next() }
}
