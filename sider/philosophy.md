# Philosophy

> *"There is an unresolved tension between the sender and recipient of information — who is to be in charge of the final form presentation? HTML clearly champions the recipient."*
> — Solvoll, Ivarsøy, Lie & Dybvik, *Telektronikk* 4/93

Bare is the answer to a question the web has spent thirty years trying to forget: **when you open a document, who is in charge — the person who wrote it, or the person reading it?**

In 1993, four researchers at Norwegian Telecom Research wrote that the young World Wide Web "championed the recipient." The reader decided how a page looked. One of those researchers, Håkon Wium Lie, would propose CSS two years later — and the balance began tilting back toward the author. Three decades on, it has tilted so far that the reader has all but disappeared beneath layout, scripts, pop-ups, and surveillance.

Bare takes the 1993 position and refuses to compromise on it: **the reader is sovereign.**

---

## The reader is sovereign

The author supplies meaning. *You* supply the presentation.

Theme, typography, line width, zoom level, whether images load at all — these are your decisions, made once and honored on every page you visit. A document author can tell you what they mean, but they cannot dictate the font you read it in, hijack your scroll, or decide that you must see their advertising to reach their words.

This is not a "reader mode" you toggle on for the rare unbearable page. In Bare it is the *only* mode. Every document, from every source, arrives in one consistent, legible form that you control.

---

## Simplicity is the winning strategy, not a sacrifice

The history of document formats is a history of simplicity defeating power.

In the early 1990s, the rich and ambitious **ODA** (Office Document Architecture) standard competed with the humble **SGML**, and SGML's simplest application — **HTML** — won the web outright. It won precisely *because* it was easy to implement and easy to read. As the 1993 paper concluded: the simplest format that is "good enough" wins, every time.

```
ODA  →  SGML  →  HTML  →  Markdown
(rich, complex)        (simple, human-readable)
```

Bare bets that the same evolution is happening again. HTML has become the new ODA: technically universal, but bloated past the point of usefulness. **Markdown is the next "good enough" format** — readable as plain text, trivial to parse, impossible to weaponize. Bare is built on that bet.

Fewer moving parts is itself the feature:

- **Fewer features** → fewer bugs → more stability
- **Less code** → faster rendering → a calmer experience
- **A smaller surface** → less to attack, less to track, less to break

---

## Privacy is the architecture, not a checkbox

Most browsers treat privacy as a setting — something you can enable, forget, misconfigure, or have silently overridden by a website. Bare treats it as a structural property that cannot be switched off, because the capabilities that enable tracking simply do not exist.

| Capability | Status in Bare | Why it matters |
|------------|----------------|----------------|
| JavaScript | Not supported | No scripts means no behavioral tracking and no malware execution |
| Cookies | Not supported | Nothing can persist a unique identifier between visits |
| Remote CSS / fonts | Not loaded | Closes the door on CSS fingerprinting and font enumeration |
| Images | Blocked by default | Tracking pixels never fire unless *you* choose to load them |
| External requests | Zero by default | One click fetches one document — and nothing else |

The Gemini protocol community calls this principle **"break all loops"**: a design where nothing a server sends can ever make its way back to that server to re-identify you. Bare applies the same logic to the whole browsing experience. There is no round trip to close, because there is no loop to begin with.

You never have to wonder whether your privacy is on. It is the only state the program can be in.

---

## Non-extensibility is a promise

The web became a surveillance platform not through any single decision, but through *extensibility*. HTML and HTTP were designed to be easy to add to, and so — feature by reasonable-sounding feature — they grew until the document-reading tool became a general-purpose computing platform that runs untrusted code on your machine by default.

Bare makes the opposite promise. There is deliberately **no plugin system, no scripting hook, no mechanism for a page to extend what the browser can do.** A document cannot ask Bare to connect somewhere else, run a computation, or store state. This is not a missing feature; it is the central guarantee. A tool that cannot be extended cannot be slowly corrupted into something that works against you.

---

## What Bare refuses — and why

Saying "no" clearly is how Bare stays true to its purpose:

- **No JavaScript.** The single largest source of tracking, fingerprinting, and attack surface on the web. Removing it removes the problem at the root.
- **No author-controlled styling.** Presentation belongs to the reader. A document with bad contrast or a hostile layout is the author's failure to impose, not yours to suffer.
- **No editing or publishing.** Bare is a reading instrument. It does one thing and gets out of the way.
- **No telemetry, ever.** Bare does not phone home. There is no "anonymized usage data," because the most private data is the data that is never collected.

These refusals are not limitations to apologize for. They are the entire point — the things that make Bare *Bare*.

---

## A different kind of internet

The Gemini FAQ puts it well: browsing should feel "more like browsing a library than wandering through a shopping mall or a casino." A library makes a world of material available and then leaves you alone with it. Nobody follows you between the shelves. Nobody redecorates the book while you read. Nobody reports your borrowing habits to a marketing department.

That is the internet Bare is trying to give back to you — one document at a time, on your terms.

---

## Related reading

- [About Bare](./about.md) — what the browser is and who it's for
- [Technology](./technology.md) — how these principles are enforced in code
- [History](./history.md) — the lineage Bare belongs to, from MultiTorg to Gemini

---

[Back to Home](../index.md)
