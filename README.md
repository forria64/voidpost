# 🕳️ Voidpost

**Anonymous document sharing on the Veilid network.**

Voidpost is a decentralized, zero-identity document sharing system built on
[Veilid](https://veilid.com) — the peer-to-peer framework from the
[Cult of the Dead Cow](https://cultdeadcow.com/),
the same beautiful maniacs who've been rattling the surveillance industry's
cage since 1984. No accounts. No servers. No tokens. No metadata breadcrumbs
for some three-letter agency to vacuum up at 3 AM. You publish a document,
you get a link, and anyone holding that link can pull it out of the ether.
The network handles the rest. You were never here.

---

## ⚡ What It Does

You drop a file into Voidpost. The system tears it apart — encrypted, chunked,
and hurled across the Veilid DHT, a distributed hash table smeared over
thousands of nodes on every continent that has electricity and an opinion.
What comes back is a share link. That link is everything: the coordinates, the
decryption key, the whole payload manifest. The publisher is hidden behind
Veilid's private routing. The reader is hidden behind safety routing. The
document itself exists simultaneously on dozens of machines owned by people
who will never know they're carrying it. Everywhere and nowhere. The way
information was always supposed to work before the landlords showed up.

---

## 🏗️ Architecture

The architecture is built on an asymmetry that most projects get catastrophically
wrong: **publishing is a commitment** — you're staking network resources to keep
data alive — while **reading is a hit-and-run** — grab the goods and vanish.
Voidpost doesn't pretend these are the same operation. It gives each one the
tool it deserves.

### 🔌 Three Clients, One Network

```mermaid
graph TD
  subgraph desktop ["🖥️ Desktop — Tauri"]
    d1["Vue 3 UI"]
    d2["Rust Backend"]
    d3["veilid-core · native"]
    d4["TCP / UDP / WS"]
    d1 --- d2 --- d3 --- d4
  end

  subgraph web ["🌐 Web Reader — WASM"]
    w1["Vue 3 UI"]
    w2["veilid-wasm"]
    w3["WebSocket only"]
    w4["Zero install"]
    w1 --- w2 --- w3 --- w4
  end

  subgraph cli ["⌨️ CLI"]
    c1["Pure Rust"]
    c2["veilid-core · native"]
    c3["TCP / UDP / WS"]
    c4["JSON output"]
    c1 --- c2 --- c3 --- c4
  end

  d4 --> VEILID["🌍 Veilid DHT Network"]
  w4 --> VEILID
  c4 --> VEILID

  style desktop fill:#0d0d0d,stroke:#ff2ecc,stroke-width:2px,color:#ff2ecc
  style web fill:#0d0d0d,stroke:#00f0ff,stroke-width:2px,color:#00f0ff
  style cli fill:#0d0d0d,stroke:#39ff14,stroke-width:2px,color:#39ff14
  style VEILID fill:#0d0d0d,stroke:#ffea00,stroke-width:3px,color:#ffea00
  style d1 fill:#1a1a1a,stroke:#ff2ecc,color:#fff
  style d2 fill:#1a1a1a,stroke:#ff2ecc,color:#fff
  style d3 fill:#1a1a1a,stroke:#ff2ecc,color:#fff
  style d4 fill:#1a1a1a,stroke:#ff2ecc,color:#fff
  style w1 fill:#1a1a1a,stroke:#00f0ff,color:#fff
  style w2 fill:#1a1a1a,stroke:#00f0ff,color:#fff
  style w3 fill:#1a1a1a,stroke:#00f0ff,color:#fff
  style w4 fill:#1a1a1a,stroke:#00f0ff,color:#fff
  style c1 fill:#1a1a1a,stroke:#39ff14,color:#fff
  style c2 fill:#1a1a1a,stroke:#39ff14,color:#fff
  style c3 fill:#1a1a1a,stroke:#39ff14,color:#fff
  style c4 fill:#1a1a1a,stroke:#39ff14,color:#fff

  linkStyle default stroke:#555,stroke-width:1px
```

| Client | Publish | Read | Refresh | Install |
|--------|---------|------|---------|---------|
| **Desktop** (Tauri) | ✅ | ✅ | ✅ System tray | Yes |
| **Web Reader** (WASM) | ❌ | ✅ | ❌ | No — just click a link |
| **CLI** | ✅ | ✅ | ✅ Daemon mode | Yes |

### 📤 Data Flow — Publish

```mermaid
flowchart LR
  A["📄 File"] --> B["Chunk"]
  B --> C["Encrypt"]
  C --> D["Store on\nVeilid DHT"]
  D --> E["🔗 Share Link"]

  style A fill:#0d0d0d,stroke:#ff2ecc,stroke-width:2px,color:#ff2ecc
  style B fill:#0d0d0d,stroke:#ff6b00,stroke-width:2px,color:#ff6b00
  style C fill:#0d0d0d,stroke:#ffea00,stroke-width:2px,color:#ffea00
  style D fill:#0d0d0d,stroke:#39ff14,stroke-width:2px,color:#39ff14
  style E fill:#0d0d0d,stroke:#00f0ff,stroke-width:2px,color:#00f0ff

  linkStyle default stroke:#888,stroke-width:2px
```

### 📥 Data Flow — Retrieve

```mermaid
flowchart LR
  A["🔗 Share Link"] --> B["Decode\nPayload"]
  B --> C["Fetch Chunks\nfrom DHT"]
  C --> D["Decrypt"]
  D --> E["Reassemble"]
  E --> F["📄 File"]

  style A fill:#0d0d0d,stroke:#00f0ff,stroke-width:2px,color:#00f0ff
  style B fill:#0d0d0d,stroke:#39ff14,stroke-width:2px,color:#39ff14
  style C fill:#0d0d0d,stroke:#ffea00,stroke-width:2px,color:#ffea00
  style D fill:#0d0d0d,stroke:#ff6b00,stroke-width:2px,color:#ff6b00
  style E fill:#0d0d0d,stroke:#ff2ecc,stroke-width:2px,color:#ff2ecc
  style F fill:#0d0d0d,stroke:#ff2ecc,stroke-width:2px,color:#ff2ecc

  linkStyle default stroke:#888,stroke-width:2px
```

The share link is a URL fragment (`#/read/<payload>`) — everything after the
`#` stays in the browser. It never touches a server. It never appears in access
logs. It never becomes evidence. The fragment is constitutionally incapable of
being logged by infrastructure you don't control.

---

## 🧰 Tech Stack

Every dependency here earned its seat. No hype-driven decisions or framework-of-the-week gambling with production stability. 🎰🚫

| Layer | Choice |
|-------|--------|
| 🌐 P2P Network | [Veilid](https://veilid.com) (v0.5.2) — DHT, private routes, safety routes |
| 🖥️ Desktop Framework | [Tauri v2](https://v2.tauri.app) — Rust backend, system webview, ~5MB binary |
| 🎨 UI Framework | Vue 3 + Composition API + TypeScript (strict) |
| 🧠 State Management | Pinia |
| 💅 Styling | Tailwind CSS |
| ⚡ Build Tool | Vite |
| 🕸️ WASM Bindings | veilid-wasm (official) |
| ⌨️ CLI Framework | clap (Rust) |
| 📦 Monorepo | pnpm workspaces (TS) + Cargo workspace (Rust) |

---

## 🗂️ Monorepo Structure

One repo. Clean lines. Every package knows its job and stays in its lane —
the kind of separation of concerns that would make a grown architect weep
quietly into their coffee.

```
voidpost/
├── packages/
│   ├── ui/              # Shared Vue 3 component library
│   ├── desktop/         # Tauri desktop app (publisher + reader)
│   ├── web/             # WASM web reader (zero-install, read-only)
│   ├── cli/             # Rust CLI (power users + automation)
│   └── core/            # Shared Rust library (veilid, crypto, chunking)
├── package.json         # pnpm workspace root
├── pnpm-workspace.yaml
├── Cargo.toml           # Rust workspace root
└── README.md
```

### 🔗 Package Dependencies

```mermaid
---
title: Package Dependencies
---
graph TD
  UI["voidpost/ui\nVue 3 Components"]
  CORE["voidpost-core\nRust Library"]
  DESKTOP["voidpost/desktop\nTauri App"]
  WEB["voidpost/web\nWASM Reader"]
  CLI["voidpost-cli\nRust Binary"]
  VEILID_CORE["veilid-core\nNative Crate"]
  VEILID_WASM["veilid-wasm\nWASM Bindings"]

  DESKTOP --> UI
  DESKTOP --> CORE
  WEB --> UI
  WEB --> VEILID_WASM
  CLI --> CORE
  CORE --> VEILID_CORE

  style UI fill:#0d0d0d,stroke:#39ff14,stroke-width:2px,color:#39ff14
  style CORE fill:#0d0d0d,stroke:#ff6b00,stroke-width:2px,color:#ff6b00
  style DESKTOP fill:#0d0d0d,stroke:#ff2ecc,stroke-width:2px,color:#ff2ecc
  style WEB fill:#0d0d0d,stroke:#00f0ff,stroke-width:2px,color:#00f0ff
  style CLI fill:#0d0d0d,stroke:#ffea00,stroke-width:2px,color:#ffea00
  style VEILID_CORE fill:#0d0d0d,stroke:#bf00ff,stroke-width:2px,color:#bf00ff
  style VEILID_WASM fill:#0d0d0d,stroke:#bf00ff,stroke-width:2px,color:#bf00ff

  linkStyle default stroke:#555,stroke-width:2px
```

---

## 🖥️ Platform Support

If it has a screen and a network stack, we'll get to it eventually. 🌍
CLI is the beachhead. The rest follows.

### 🖥️ Desktop (Tauri v2)
- 🐧 Linux (.deb, .AppImage, .rpm)
- 🍎 macOS (.dmg — signed + notarized)
- 🪟 Windows (.msi, .exe — signed )
- 🤖 Android (.apk — future)
- 📱 iOS (future)

### 🌐 Web Reader
- 🏄 Any modern browser with WASM support

### ⌨️ CLI
- 🛠️ Linux, macOS, Windows (prebuilt binaries + `cargo install`)

---

## 🛡️ Privacy Model

🔐 Privacy is not a feature in Voidpost. It is the architecture. Strip it out
and there is nothing left — no app, no protocol, no reason to exist. Every
design decision flows downstream from one principle: **the system must not
be capable of betraying its users, even under duress.**

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': {'actorBkg': '#0d0d0d', 'actorBorder': '#ff2ecc', 'actorTextColor': '#ff2ecc', 'actorLineColor': '#555', 'signalColor': '#00f0ff', 'signalTextColor': '#00f0ff', 'noteBkgColor': '#1a1a1a', 'noteTextColor': '#39ff14', 'noteBorderColor': '#39ff14', 'activationBkgColor': '#1a1a1a', 'activationBorderColor': '#ffea00', 'sequenceNumberColor': '#ffea00'}}}%%
sequenceDiagram
  participant P as 🖥️ Publisher
  participant H1 as Hop 1
  participant H2 as Hop 2
  participant DHT as 🌍 Veilid DHT
  participant H3 as Hop 3
  participant H4 as Hop 4
  participant R as 🌐 Reader

  rect rgba(255,46,204,0.1)
    Note over P,H2: 🔒 Private Route (sender anonymity)
    P->>H1: encrypted
    H1->>H2: encrypted
    H2->>DHT: write chunks
  end

  rect rgba(0,240,255,0.1)
    Note over H3,R: 🔒 Safety Route (receiver anonymity)
    DHT->>H3: read chunks
    H3->>H4: encrypted
    H4->>R: encrypted
  end

  Note over P,R: ⚡ Publisher and Reader never see each other.
```

- 👻 **No accounts, no identity, no tokens.** There is nothing to link, nothing
  to subpoena, nothing to hand over in a conference room with bad lighting
  and worse intentions.
- 🕳️ **Private Routes** — The publisher's node identity is severed from DHT writes.
  Your operations bounce through multiple relay hops before they touch the
  hash table. Your IP never shares a zip code with your data.
- 🛤️ **Safety Routes** — The reader gets the same treatment in reverse.
  Pull a document off the DHT and your node ID is nowhere near the request.
- 🔗 **URL fragments** — The share link's payload lives after the `#`. Browsers
  do not send fragments to servers. Period. Not in headers, not in referrers,
  not in any log that any sysadmin on earth will ever read.
- 🚫 **Zero telemetry** — No analytics. No phone-home. No clever "anonymous
  usage metrics" that always turn out to be neither anonymous nor metric.
  The only packets leaving your machine are Veilid protocol.
- 🔒 **Encrypted at rest** — Documents are ciphertext before they ever touch the
  DHT. Node operators, relay operators, network observers — they all see the
  same thing: noise. Beautiful, uninterpretable, plausibly-deniable noise. 📡

---

## 🐄 Why Veilid?

Because every other option has a fatal flaw and we're tired of pretending
otherwise. 🪦

Veilid is a pure infrastructure protocol — no blockchain, no token, no
financialized incentive structure that turns every participant into a day
trader with a node. It offers encrypted P2P routing and distributed storage
as a public utility, the way the internet was supposed to work before
venture capital got its hooks into the protocol layer.

LBRY tried the token play and the SEC gutted them in open court — a $22M
fine and a full shutdown, because when you issue a token, you've handed
regulators the exact weapon they need to destroy you. Tor works, but it was
built for anonymizing streams, not distributing content. IPFS has distributed
storage but zero native anonymity — your node announces what you're hosting
to anyone who asks. GNUnet has been academically promising since 2001 and
will be academically promising when the sun burns out.

Veilid is the convergence point: anonymous routing + distributed storage +
no legal attack surface from token economics. Built by people who understand
that the most important feature of a privacy tool is not getting shut down.

---

*"🕳️ Voidpost. 🔐 Encrypted at birth. 👻 Anonymous by design. 💨 Gone when you're done."*
