// Writes the README figures: anatomy-two-key-closure.svg and anatomy-logos-stack.svg
//   node scripts/anatomy/build.mjs
// Needs the anatomy skill kit (isometric SVG helpers). Point ANATOMY_KIT at iso-kit.mjs if it lives elsewhere.
import { writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { homedir } from "node:os";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = join(HERE, "../../public/images");
const k = await import(process.env.ANATOMY_KIT || join(homedir(), ".claude/skills/anatomy/kit/iso-kit.mjs"));

const W = 720;
const ACC = "#ed7b58";
const CSS = `${k.ISO_CSS}
.iso{--anatomy-card:#171717;--anatomy-paper:#1c1c1c;--anatomy-face:#262626;--anatomy-top:#2f2f2f;
--anatomy-shade-0:#1c1c1c;--anatomy-shade-1:#212121;--anatomy-shade-2:#262626;--anatomy-shade-3:#2b2b2b;
--anatomy-lit-top:#2f2f2f;--anatomy-lit-shade:#262626;--anatomy-hi:#a4a4a4;--anatomy-mid:#6b6b6b;--anatomy-lo:#434343;
--anatomy-faint:#2f2f2f;--anatomy-lit:${ACC};--anatomy-halo:#000;--anatomy-halo-opacity:.55;--anatomy-dot:#d0d0d0;--anatomy-weight:.8px}
.iso-svg{width:auto;height:auto;overflow:hidden}
.iso-solid.acc>.iso-fill{fill:#b9553a}.iso-solid.acc>.iso-shade[data-shade="0"]{fill:#9b472f}.iso-solid.acc>.iso-shade[data-shade="1"]{fill:#ad4f35}
.iso-solid.acc>.iso-shade[data-shade="2"]{fill:#c85d3e}.iso-solid.acc>.iso-shade[data-shade="3"]{fill:#dc6d4b}.iso-solid.acc>.iso-top{fill:${ACC}}
.iso-solid.acc>.iso-edge{stroke:#5d2a1a}.iso-solid.acc>.iso-crease{stroke:#7e3a25}
.iso-solid.grn>.iso-fill{fill:#3f9d73}.iso-solid.grn>.iso-shade{fill:#4fbd8a}.iso-solid.grn>.iso-top{fill:#6fe0a8}.iso-solid.grn>.iso-edge{stroke:#1f5a40}
.iso-line[data-tone="acc"]{stroke:${ACC}}
.glow{fill:#6fe0a8;opacity:.28;filter:blur(7px)}
.card{fill:#171717;stroke:#2f2f2f;stroke-width:1}
.txt{font-family:ui-sans-serif,system-ui,-apple-system,"Segoe UI",Helvetica,Arial,sans-serif;font-size:13px}
.cap{fill:#a4a4a4;font-size:12px;letter-spacing:.08em;text-transform:uppercase}
.ttl{fill:#fff;font-size:12.5px;font-weight:600;letter-spacing:.08em;text-transform:uppercase}
.pill{fill:#262626;stroke:#434343;stroke-width:1}.pill-t{fill:#fff;font-weight:600}
.pill[data-acc]{stroke:${ACC}}
`;

const S = (p, o) => k.solidSvg(p, o);
const L = (d, o) => k.lineSvg(d, o);
const D = (pts, o) => k.dotsSvg(pts, o);
const tenth = (v) => Math.round(v * 10) / 10;

function frame({ H, fig, title, hint, label, body }) {
  return `<svg xmlns="http://www.w3.org/2000/svg" class="iso-svg iso" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" role="img" aria-label="${label}"><title>${title}</title><style>${CSS}</style>
<rect class="card" x=".5" y=".5" width="${W - 1}" height="${H - 1}" rx="12"/>
<text class="txt cap" x="22" y="30">${fig}</text><text class="txt ttl" x="${W - 22}" y="30" text-anchor="end">${title}</text>
<text class="txt cap" x="22" y="${H - 20}" style="text-transform:none;letter-spacing:.02em;font-size:13px">${hint}</text>
${body}</svg>`;
}

// A label tag with a thin wire whose dot lands on the part. `at` is the part point on screen, `c` the tag centre.
function tag(text, c, at, { acc = false, from = "auto" } = {}) {
  const w = Math.round(text.length * 7.3 + 20), h = 24;
  const [cx, cy] = c;
  const sides = { l: [cx - w / 2, cy], r: [cx + w / 2, cy], t: [cx, cy - h / 2], b: [cx, cy + h / 2] };
  const pick = from !== "auto" ? from : Math.abs(at[0] - cx) > Math.abs(at[1] - cy) ? (at[0] > cx ? "r" : "l") : at[1] > cy ? "b" : "t";
  const wire = k.wireSvg(sides[pick], at, { tone: acc ? "acc" : "hi", ends: "to", size: 1.8 });
  return `${wire}<rect class="pill" ${acc ? 'data-acc="true"' : ""} x="${tenth(cx - w / 2)}" y="${tenth(cy - h / 2)}" width="${w}" height="${h}" rx="7"/><text class="txt pill-t" x="${cx}" y="${cy + 4.6}" text-anchor="middle">${text}</text>`;
}

// ---------------------------------------------------------------- Figure 1: two keys to close a fault
function figure1() {
  const H = 470;
  const BENCH = { x: -112, y: -152, w: 224, d: 304, r: 9 }, BH = 10, FOOT = 5;
  const P = k.fitProjection([...k.boxCorners(BENCH, -BH - FOOT, 0), [-84, -66, 62], [0, 40, 60], [70, 100, 8]], W, H - 6, { pad: 14, headroom: 40, azimuth: 45 });
  const at = (p) => k.iso(p, P);
  const box = (plan, z, h, tone = "mid", o = {}) => S(k.slabOf(plan, z, h, P, o.steps ?? 6, o.bevel ?? 0.6), { tone, ...o.style });
  const rr = (x, y, w, d, r = 1.5) => ({ x, y, w, d, r });
  const line = (pts, z, tone = "lo", o = {}) => L(k.onTop(pts, z, P, o.closed), { tone, ...o.style });
  const dots = (pts, z, size = 0.5, tone = "mid") => D(pts.map(([x, y]) => at([x, y, z])), { size, tone });
  const byDepth = (items) => [...items].sort((a, b) => k.depthOf([a.x, a.y, 0], P) - k.depthOf([b.x, b.y, 0], P));

  // --- bench
  const bench = () => {
    const o = [`<path class="iso-halo" d="${k.haloOf(BENCH, -BH - FOOT, BH, P)}"/>`];
    for (const [x, y] of [...k.corners(BENCH, 20)].sort((a, b) => k.depthOf([a[0], a[1], 0], P) - k.depthOf([b[0], b[1], 0], P))) {
      o.push(S(k.cylinder(x, y, 7, -BH - FOOT, 1.6, P, 24), { tone: "lo" }), S(k.cylinder(x, y, 5.4, -BH - FOOT + 1.6, FOOT - 1.6, P, 24), { tone: "mid" }));
    }
    o.push(S(k.slabOf(BENCH, -BH, BH, P, 8, 1.4), { tone: "mid" }));
    o.push(L(k.planOutline(k.insetPlan(BENCH, 5), 0, P), { tone: "lo" }));
    const sc = k.corners(BENCH, 11);
    o.push(L(sc.map(([x, y]) => k.ring(x, y, 2.2, 0, P, 14)).join(""), { tone: "lo" }), D(sc.map(([x, y]) => at([x, y, 0])), { size: 0.5 }));
    // ruler strip along the back edge and a label plate with three rules at the front
    const strip = rr(-92, -141, 8, 112, 1);
    o.push(S(k.slabOf(strip, 0, 0.6, P, 3), { tone: "lo", crease: "none" }));
    const t = k.topTicks(strip.y + 3, strip.y + strip.d - 3, 4, 5, strip.x + 1, 0.6, [2, 3.4], P, "y");
    o.push(L(t.minor, { tone: "lo" }), L(t.major, { tone: "mid" }));
    const plate = rr(88, 118, 14, 28, 2);
    o.push(S(k.slabOf(plate, 0, 0.8, P, 4), { tone: "lo", crease: "none" }));
    for (const dx of [3.5, 7, 10.5]) o.push(line([[plate.x + dx, plate.y + 4], [plate.x + dx, plate.y + plate.d - 4]], 0.8, "mid"));
    return o.join("");
  };

  // --- floor: bays, rails, sleepers, chevrons
  const BAY = { open: 95, wait: 20, done: -105 };
  const floor = () => {
    const o = [];
    for (const [name, c] of Object.entries(BAY)) {
      o.push(S(k.slabOf(rr(-34, c - 33, 68, 66, 4), 0, 0.8, P, 4), { tone: "lo", crease: "none" }));
      o.push(L(k.planOutline(rr(-30, c - 29, 60, 58, 3), 0.8, P, 4), { tone: name === "done" ? "hi" : "faint" }));
    }
    for (const y of [60, -22, -36, -62]) o.push(S(k.slabOf(rr(-36, y - 2.5, 72, 5, 1), 0, 1.2, P, 3, 0.3), { tone: "lo", crease: "none" }));
    for (const x of [-28, 24]) {
      o.push(S(k.slabOf(rr(x, -140, 4, 266, 1), 0, 4, P, 3, 0.4), { tone: "mid" }));
      o.push(dots([[x + 2, -134], [x + 2, 118]], 4, 0.4, "mid"));
    }
    // ghost of the box in the Open bay, and where it will rest when resolved
    o.push(L(k.planOutline(rr(-22, 70, 44, 44, 3), 0.8, P, 4), { tone: "mid", dashed: true }));
    o.push(L(k.planOutline(rr(-22, -126, 44, 44, 3), 0.8, P, 4), { tone: "hi", dotted: true }));
    // chevrons: Open -> Waiting is open to stewards, Waiting -> Resolved is behind the bar
    for (const y of [60, 53]) o.push(line([[-8, y + 5], [0, y - 3], [8, y + 5]], 0.8, "hi"));
    for (const y of [-72, -79]) o.push(line([[-8, y + 5], [0, y - 3], [8, y + 5]], 0.8, "lo"));
    return o.join("");
  };

  // --- keys & barrels (vertical keys on top of consoles)
  const key = ({ x, y, z, tone = "mid", acc = false, lift = 0, along = "x" }) => {
    const o = [];
    const z0 = z + lift;
    const cls = acc ? "acc" : undefined;
    const st = { tone: acc ? "hi" : tone, className: cls };
    o.push(S(k.cylinder(x, y, 1.5, z0, 8, P, 12), { ...st, crease: "none" }));
    if (lift) o.push(S(k.slabOf(rr(x - 1.5, y - 1.5, 5, 3, 0.5), z0 - 0.2, 3.2, P, 2, 0), { ...st, crease: "none" }));
    o.push(S(k.cylinder(x, y, 2.6, z0 + 8, 2, P, 14), { ...st }));
    const bow = along === "x" ? rr(x - 6, y - 3.8, 12, 7.6, 3.8) : rr(x - 3.8, y - 6, 7.6, 12, 3.8);
    o.push(S(k.slabOf(bow, z0 + 10, 2.6, P, 6, 0.5), { ...st }));
    o.push(L(k.ring(along === "x" ? x + 2.6 : x, along === "x" ? y : y + 2.6, 1.7, z0 + 12.6, P, 12), { tone: acc ? "lo" : "mid" }));
    return o.join("");
  };
  const barrel = (x, y, z, r) => [
    S(k.cylinder(x, y, r, z, 2.6, P, 24, 0.4), { tone: "mid" }),
    L(k.ring(x, y, r - 1.7, z + 2.6, P, 20), { tone: "lo" }),
    L(k.lineOnTop([x - r * 0.55, y], [x + r * 0.55, y], z + 2.6, P), { tone: "hi" }),
  ].join("");

  // --- lock console (the lock to "resolved")
  const CON = rr(-104, -86, 70, 76, 4), CZ = 34;
  const console_ = () => {
    const o = [`<path class="iso-halo" d="${k.haloOf(CON, 0, CZ, P)}"/>`];
    o.push(S(k.slabOf(CON, 0, CZ, P, 6, 0.8), { tone: "mid" }));
    // front faces: seams and vents (lines run edge to edge)
    const fx = CON.x + CON.w, fy = CON.y + CON.d;
    for (const z of [8, 24]) {
      o.push(L(k.segment([CON.x + 3, fy, z], [fx - 3, fy, z], P), { tone: "faint" }));
      o.push(L(k.segment([fx, CON.y + 3, z], [fx, fy - 3, z], P), { tone: "faint" }));
    }
    for (const x of [CON.x + 10, CON.x + 22, CON.x + 34, CON.x + 46]) o.push(L(k.rise([x, fy], 10, 22, P), { tone: "lo" }));
    o.push(D([[CON.x + 5, fy, 5], [fx - 5, fy, 5], [CON.x + 5, fy, 29], [fx - 5, fy, 29]].map(at), { size: 0.5 }));
    o.push(D([[fx, CON.y + 5, 5], [fx, fy - 5, 5], [fx, CON.y + 5, 29], [fx, fy - 5, 29]].map(at), { size: 0.5 }));
    o.push(S(k.slabOf(k.insetPlan(CON, 3), CZ, 2, P, 5, 0.4), { tone: "mid", crease: "faint" }));
    return o.join("");
  };
  const consoleTop = () => {
    const Z = CZ + 2;
    const R = [-84, -66], C = [-54, -66], A = [-84, -26], B = [-58, -26];
    const o = [];
    o.push(S(k.slabOf(rr(-91, -33, 40, 14, 7), Z, 0.8, P, 6), { tone: "lo", crease: "none" })); // yoke: both must turn
    o.push(line([[-84, -26], [-58, -26]], Z + 0.8, "hi"));
    const parts = byDepth([
      { x: R[0], y: R[1], svg: barrel(R[0], R[1], Z, 8.5) + key({ x: R[0], y: R[1], z: Z + 2.6, acc: true, along: "x" }) },
      { x: C[0], y: C[1], svg: barrel(C[0], C[1], Z, 6.5) + k.lineSvg(k.segment([C[0], C[1], Z + 2.6], [C[0], C[1], Z + 2.6 + 13], P), { tone: "mid", dotted: true }) + key({ x: C[0], y: C[1], z: Z + 2.6, tone: "mid", lift: 13, along: "x" }) },
      { x: A[0], y: A[1], svg: barrel(A[0], A[1], Z, 5.5) + key({ x: A[0], y: A[1], z: Z + 2.6, tone: "hi", along: "y" }) },
      { x: B[0], y: B[1], svg: barrel(B[0], B[1], Z, 5.5) + key({ x: B[0], y: B[1], z: Z + 2.6, tone: "hi", along: "y" }) },
    ]);
    return o.join("") + parts.map((p) => p.svg).join("");
  };

  // --- bar (the gate) and catch post
  const bar = () => {
    const o = [];
    o.push(S(k.slabOf(rr(-34, -49, 68, 8, 1.5), 14, 6, P, 3, 0.4), { tone: "hi" }));
    for (let x = -26; x <= 26; x += 13) o.push(L(k.lineOnTop([x, -49], [x, -41], 20, P), { tone: "lo" }));
    o.push(S(k.slabOf(rr(34, -52, 11, 14, 2), 0, 30, P, 4, 0.6), { tone: "mid" }));
    o.push(L(k.rise([34, -41], 14, 20, P), { tone: "faint" }));
    o.push(D([at([45, -45, 10]), at([45, -45, 22]), at([39, -38, 26])], { size: 0.45 }));
    return o.join("");
  };

  // --- steward console, link rod, pusher
  const SC = rr(-96, 104, 44, 34, 3);
  const steward = () => {
    const o = [`<path class="iso-halo" d="${k.haloOf(SC, 0, 22, P)}"/>`];
    o.push(S(k.slabOf(SC, 0, 22, P, 5, 0.8), { tone: "mid" }));
    const fx = SC.x + SC.w, fy = SC.y + SC.d;
    o.push(L(k.segment([SC.x + 3, fy, 8], [fx - 3, fy, 8], P), { tone: "faint" }), L(k.segment([fx, SC.y + 3, 8], [fx, fy - 3, 8], P), { tone: "faint" }));
    o.push(D([[SC.x + 5, fy, 4], [fx - 5, fy, 4], [fx, SC.y + 5, 4], [fx, fy - 5, 4]].map(at), { size: 0.5 }));
    o.push(S(k.slabOf(k.insetPlan(SC, 3), 22, 1.6, P, 4, 0.3), { tone: "mid" }));
    o.push(barrel(-74, 121, 23.6, 7) + key({ x: -74, y: 121, z: 23.6 + 2.6, tone: "mid", along: "x" }));
    return o.join("");
  };
  const pusher = () => [
    S(k.slabOf(rr(-52, 119, 34, 4, 1), 6, 4, P, 3, 0.3), { tone: "mid" }),
    S(k.slabOf(rr(-20, 112, 40, 9, 2), 2, 12, P, 4, 0.5), { tone: "hi" }),
    D([at([-20, 118, 12]), at([-20, 118, 5])], { size: 0.4 }),
  ].join("");

  // --- the box: a report, with a dripping tap
  const box_ = () => {
    const o = [`<path class="iso-halo" d="${k.haloOf(rr(-30, -6, 60, 52, 3), 4, 3, P)}" style="filter:blur(4px)"/>`];
    o.push(S(k.slabOf(rr(-30, -6, 60, 52, 3), 4, 3, P, 4, 0.5), { tone: "lo" }));
    o.push(S(k.slabOf(rr(-22, -2, 44, 44, 3), 7, 20, P, 5, 0.7), { tone: "mid" }));
    o.push(S(k.slabOf(rr(-23, -3, 46, 46, 3.5), 27, 4, P, 5, 0.7), { tone: "hi" }));
    // straps and hasp on the +y face
    o.push(S(k.slabOf(rr(-14, -3.4, 5, 46.8, 1), 31, 0.5, P, 2), { tone: "lo", crease: "none" }), S(k.slabOf(rr(9, -3.4, 5, 46.8, 1), 31, 0.5, P, 2), { tone: "lo", crease: "none" }));
    o.push(S(k.slabOf(rr(-5, 42, 10, 1.6, 0.4), 15, 12, P, 2, 0.2), { tone: "lo", crease: "none" }));
    o.push(S(k.slabOf(rr(-4, 43.6, 8, 3.6, 1), 10, 8, P, 3, 0.3), { tone: "hi" }));
    o.push(L(k.sideRing([0, 43.6, 18.4], 2.6, "x", P, 14), { tone: "mid" }));
    o.push(D([at([-9, 45, 24]), at([9, 45, 24]), at([22, 12, 22]), at([22, 30, 22])], { size: 0.45 }));
    // tap on the lid, dripping
    o.push(S(k.cylinder(0, 6, 5, 31, 2, P, 20, 0.3), { tone: "mid" }));
    o.push(S(k.cylinder(0, 6, 2.2, 33, 13, P, 14), { tone: "hi", crease: "none" }));
    o.push(S(k.slabOf(rr(-6, 4.8, 12, 2.4, 1.2), 46, 2, P, 3, 0.2), { tone: "mid" }));
    o.push(S(k.slabOf(rr(-2, 4, 4, 15, 2), 40, 4, P, 4, 0.3), { tone: "hi" }));
    o.push(S(k.cylinder(0, 17, 2.2, 36, 4, P, 12), { tone: "hi", crease: "none" }));
    o.push(L(k.ring(0, 17, 3.6, 31, P, 16), { tone: "acc" }) + L(k.ring(0, 17, 6.2, 31, P, 20), { tone: "acc", dashed: true }));
    o.push(S(k.cylinder(0, 17, 1.4, 33.2, 2.4, P, 10), { tone: "hi", className: "acc", crease: "none" }));
    return o.join("");
  };

  // --- Resolved bay lamp
  const lamp = () => {
    const c = at([-24, -128, 34]);
    return `<ellipse class="glow" cx="${c[0]}" cy="${c[1]}" rx="14" ry="12"/>` +
      S(k.cylinder(-24, -128, 3, 0, 28, P, 14), { tone: "mid", crease: "none" }) +
      S(k.cylinder(-24, -128, 6.4, 28, 3, P, 16), { tone: "mid" }) +
      S(k.cylinder(-24, -128, 5, 31, 6, P, 16, 0.3), { tone: "hi", className: "grn" });
  };

  // --- Reopen lane (reporter's way back)
  const lane = () => {
    const o = [];
    o.push(S(k.slabOf(rr(56, -125, 20, 228, 4), 0, 3, P, 4, 0.5), { tone: "mid" }));
    o.push(L(k.planOutline(rr(59, -122, 14, 222, 3), 3, P, 3), { tone: "lo" }));
    for (let y = -108; y <= 94; y += 24) o.push(line([[60, y - 4], [66, y + 4], [72, y - 4]], 3, "acc", { style: { } }));
    for (const c of [-105, 95]) {
      o.push(S(k.slabOf(rr(34, c - 3, 22, 6, 1), 0, 2.2, P, 3, 0.3), { tone: "lo" }));
      o.push(line([[38, c], [52, c]], 2.2, "acc"));
    }
    o.push(D([at([66, -118, 3]), at([66, 98, 3])], { size: 0.45 }));
    return o.join("");
  };

  const body = [
    bench(), floor(),
    console_(), consoleTop(),
    bar(),
    lane(),
    steward(), pusher(),
    lamp(), box_(),
  ];
  // order note: lane sits at larger x than the box and bar, so it is drawn after them
  const ordered = [bench(), floor(), console_(), consoleTop(), bar(), lamp(), box_(), lane(), steward(), pusher()];
  void body;

  const labels = [
    tag("Fix claimed", [112, 380], at([-74, 121, 38]), { from: "t" }),
    tag("Waiting", [140, 300], at([0, 43.6, 14]), { from: "r" }),
    tag("Reporter", [300, 52], at([-84, -66, 49]), { acc: true }),
    tag("2 residents", [452, 62], at([-71, -26, 49]), {}),
    tag("Claimant: no fit", [112, 98], at([-54, -66, 52]), {}),
    tag("Resolved", [380, 178], at([-24, -128, 38])),
    tag("Reopen", [634, 340], at([66, 20, 3]), { acc: true }),
  ];
  return frame({ H, fig: "Fig 1", title: "Two keys to close a fault", hint: "A claim is not a closure.", label: "An isometric strongbox holding a dripping-tap report. The steward's key only moves it to a waiting slot. The lock to resolved opens for the reporter's key, or two resident keys together. The claimant's key does not fit. A return lane lets the reporter reopen.", body: ordered.join("") + labels.join("") });
}

writeFileSync(join(OUT, "anatomy-two-key-closure.svg"), figure1());
console.log("wrote anatomy-two-key-closure.svg");

// ---------------------------------------------------------------- Figure 2: the Logos stack under Pukaar
function figure2() {
  const H = 470;
  const P = k.fitProjection([
    ...k.boxCorners({ x: -118, y: -74, w: 236, d: 148, r: 8 }, -10, 0),
    [-92, -52, 72], [92, -52, 72], [-92, 52, 72], [92, 52, 72],
  ], W, H - 8, { pad: 20, headroom: 42, azimuth: 45 });
  const at = (p) => k.iso(p, P);
  const rr = (x, y, w, d, r = 6) => ({ x, y, w, d, r });
  const slab = (plan, z, h, tone = "mid", o = {}) => S(k.slabOf(plan, z, h, P, o.steps ?? 6, o.bevel ?? 0.8), { tone, ...o.style });
  const line = (pts, z, tone = "lo", o = {}) => L(k.onTop(pts, z, P, o.closed), { tone, ...o.style });
  const dots = (pts, z, size = 0.55, tone = "mid") => D(pts.map(([x, y]) => at([x, y, z])), { size, tone });

  const base = rr(-118, -74, 236, 148, 8);
  const stack = [
    { name: "LEZ", plan: rr(-82, -42, 164, 84, 5), z: 0, h: 13, tone: "lo", accent: false },
    { name: "Storage", plan: rr(-76, -38, 152, 76, 5), z: 16, h: 18, tone: "mid", accent: false },
    { name: "Delivery", plan: rr(-70, -34, 140, 68, 5), z: 37, h: 20, tone: "mid", accent: true },
    { name: "Chat", plan: rr(-64, -30, 128, 60, 5), z: 60, h: 20, tone: "hi", accent: false },
    { name: "Pukaar", plan: rr(-58, -26, 116, 52, 5), z: 83, h: 25, tone: "hi", accent: true },
  ];

  const body = [];
  body.push(`<path class="iso-halo" d="${k.haloOf(base, -10, 0, P)}"/>`);
  body.push(slab(base, -10, 10, "lo", { bevel: 1.5 }));
  body.push(L(k.planOutline(k.insetPlan(base, 6), 0, P), { tone: "faint" }));
  for (const [x, y] of k.corners(base, 12)) {
    body.push(S(k.cylinder(x, y, 4.5, -14, 4, P, 18), { tone: "mid" }));
    body.push(L(k.ring(x, y, 2.4, -10, P, 14), { tone: "lo" }));
  }
  for (const layer of stack) {
    body.push(slab(layer.plan, layer.z, layer.h, layer.tone, {
      style: layer.accent ? { className: "acc" } : {},
      bevel: 1.1,
    }));
    body.push(L(k.planOutline(k.insetPlan(layer.plan, 5), layer.z + layer.h, P), { tone: layer.accent ? "hi" : "lo" }));
    for (const [x, y] of k.corners(layer.plan, 8)) body.push(D([at([x, y, layer.z + layer.h])], { size: 0.42, tone: layer.accent ? "hi" : "mid" }));
    for (const x of [-layer.plan.w * 0.25, 0, layer.plan.w * 0.25]) body.push(line([[x, layer.plan.y + 8], [x, layer.plan.y + layer.plan.d - 8]], layer.z + layer.h + 0.3, "faint"));
  }

  // A small vertical data spine with connector collars and directional ticks.
  const spineX = 0, spineY = 58;
  body.push(S(k.cylinder(spineX, spineY, 3.2, 10, 86, P, 16), { tone: "hi", crease: "none" }));
  for (const z of [16, 37, 60, 83]) {
    body.push(S(k.cylinder(spineX, spineY, 5.5, z, 2, P, 18), { tone: "mid" }));
    body.push(L(k.ring(spineX, spineY, 4, z + 2, P, 18), { tone: "lo" }));
  }
  for (const z of [25, 46, 69, 92]) body.push(line([[-22, 58], [22, 58]], z, "acc"));
  body.push(dots([[-44, 50], [-30, 50], [30, 50], [44, 50]], 102, 0.7, "hi"));

  const labels = [
    tag("Pukaar", [120, 90], at([58, -26, 101]), { acc: true, from: "r" }),
    tag("Chat", [114, 154], at([64, -30, 70]), { from: "r" }),
    tag("Delivery", [108, 218], at([70, -34, 47]), { acc: true, from: "r" }),
    tag("Storage", [110, 284], at([76, -38, 23]), { from: "r" }),
    tag("LEZ", [116, 350], at([82, -42, 6]), { from: "r" }),
  ];
  return frame({
    H,
    fig: "Fig 2",
    title: "Built on Logos",
    hint: "The app keeps the record. The modules carry and anchor it.",
    label: "An isometric stack showing Pukaar on top of Chat, Delivery, Storage and the LEZ chain, with a shared data spine.",
    body: body.join("") + labels.join(""),
  });
}

writeFileSync(join(OUT, "anatomy-logos-stack.svg"), figure2());
console.log("wrote anatomy-logos-stack.svg");
