/// template.rs — Full HTML page template (dark theme, same design as original).
///
/// Placeholders (replaced at runtime):
///   __CONTENT__      → rendered HTML body
///   __FILENAME__     → file basename (HTML-escaped)
///   __MERMAID_SRC__  → path or URL to mermaid.min.js
///   __WORD_COUNT__   → integer word count
///   __READ_TIME__    → integer minutes read time

pub const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"/>
<title>__FILENAME__ — MD Reader</title>
<style>
  :root {
    --bg: #0a0b0f;
    --surface: #12141a;
    --surface2: #1a1d26;
    --border: #252830;
    --accent: #5eead4;
    --accent2: #fbbf24;
    --accent-glow: rgba(94,234,212,0.12);
    --text: #e4e6ed;
    --text-secondary: #8b8fa3;
    --muted: #5a5e72;
    --code-bg: #0d0f14;
    --heading: #f0f2f8;
    --sidebar-w: 280px;
    --topbar-h: 52px;
    --radius: 10px;
    --t: 0.22s cubic-bezier(0.4,0,0.2,1);
  }
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  html { scroll-behavior: smooth; }
  body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 16px; line-height: 1.72;
    color: var(--text); background: var(--bg);
    -webkit-font-smoothing: antialiased;
    overflow: hidden; height: 100vh; height: 100dvh;
  }

  /* ── Scrollbars ── */
  ::-webkit-scrollbar { width: 5px; height: 5px; }
  ::-webkit-scrollbar-track { background: transparent; }
  ::-webkit-scrollbar-thumb { background: var(--border); border-radius: 99px; }
  ::-webkit-scrollbar-thumb:hover { background: var(--muted); }

  /* ── Layout ── */
  .app { display: flex; height: 100vh; height: 100dvh; overflow: hidden; }

  /* ── Sidebar ── */
  .sidebar {
    width: var(--sidebar-w); min-width: var(--sidebar-w);
    background: var(--surface); border-right: 1px solid var(--border);
    display: flex; flex-direction: column; overflow: hidden;
    transition: width var(--t), min-width var(--t), border-color var(--t);
    z-index: 100;
  }
  .sidebar.collapsed { width: 0; min-width: 0; border-color: transparent; }

  .sidebar-header {
    padding: 18px 20px 14px; border-bottom: 1px solid var(--border); flex-shrink: 0;
  }
  .sidebar-logo {
    display: flex; align-items: center; gap: 10px;
    font-size: 0.72rem; font-weight: 700; letter-spacing: 0.16em;
    text-transform: uppercase; color: var(--accent);
  }
  .sidebar-file {
    padding: 14px 20px 12px; border-bottom: 1px solid var(--border); flex-shrink: 0;
  }
  .sidebar-file-label {
    font-size: 0.62rem; font-weight: 600; letter-spacing: 0.12em;
    text-transform: uppercase; color: var(--muted); margin-bottom: 4px;
  }
  .sidebar-file-name {
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    font-size: 0.72rem; color: var(--accent2); word-break: break-all; line-height: 1.4;
  }
  .toc-header {
    padding: 14px 20px 8px; font-size: 0.62rem; font-weight: 600;
    letter-spacing: 0.12em; text-transform: uppercase; color: var(--muted);
  }
  .toc { overflow-y: auto; flex: 1; padding-bottom: 0; }
  .sidebar-footer {
    padding: 10px 20px; border-top: 1px solid var(--border);
    font-size: 0.6rem; color: var(--muted); flex-shrink: 0;
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    letter-spacing: 0.04em;
  }
  .toc a {
    display: block; padding: 5px 20px;
    font-size: 0.78rem; color: var(--text-secondary);
    text-decoration: none; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    border-left: 2px solid transparent;
    transition: color var(--t), background var(--t), border-color var(--t);
  }
  .toc a:hover { color: var(--text); background: var(--accent-glow); }
  .toc a.active { color: var(--accent); border-left-color: var(--accent); background: var(--accent-glow); }
  .toc a.h2 { padding-left: 32px; }
  .toc a.h3 { padding-left: 46px; font-size: 0.73rem; }
  .toc a.h4 { padding-left: 60px; font-size: 0.7rem; color: var(--muted); }

  /* ── Main area ── */
  .main { flex: 1; display: flex; flex-direction: column; overflow: hidden; min-width: 0; }

  /* ── Topbar ── */
  .topbar {
    height: var(--topbar-h); min-height: var(--topbar-h);
    background: var(--surface); border-bottom: 1px solid var(--border);
    display: flex; align-items: center; gap: 10px;
    padding: 0 16px; flex-shrink: 0;
  }
  .topbar-toggle {
    background: none; border: none; cursor: pointer;
    color: var(--text-secondary); padding: 7px; border-radius: 8px;
    display: flex; align-items: center;
    transition: color var(--t), background var(--t);
    flex-shrink: 0;
  }
  .topbar-toggle:hover { color: var(--text); background: var(--surface2); }
  .topbar-title {
    font-size: 0.83rem; font-weight: 500; color: var(--text);
    flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .topbar-badge {
    font-family: 'JetBrains Mono', monospace; font-size: 0.62rem;
    padding: 3px 9px; background: var(--accent-glow); color: var(--accent);
    border: 1px solid rgba(94,234,212,0.18); border-radius: 99px; white-space: nowrap;
  }
  .topbar-actions { display: flex; gap: 5px; }
  .btn-icon {
    background: none; border: 1px solid var(--border); cursor: pointer;
    color: var(--text-secondary); padding: 5px 10px; border-radius: 8px;
    display: flex; align-items: center; gap: 5px;
    font-size: 0.72rem; font-family: inherit; font-weight: 500;
    transition: color var(--t), background var(--t), border-color var(--t);
  }
  .btn-icon:hover { color: var(--text); border-color: var(--muted); background: var(--surface2); }

  /* ── Progress bar ── */
  .progress-bar { height: 2px; background: var(--border); flex-shrink: 0; }
  .progress-fill {
    height: 100%; width: 0%;
    background: linear-gradient(90deg, var(--accent), var(--accent2));
    transition: width 0.08s ease-out;
  }

  /* ── Content ── */
  .content-wrap {
    flex: 1; overflow-y: auto; overflow-x: hidden;
    padding: 48px max(24px, calc((100% - 820px) / 2)) 72px;
    scroll-behavior: smooth;
  }
  .md-body { max-width: 820px; margin: 0 auto; animation: fadeIn 0.35s ease-out; }

  /* ── Typography ── */
  .md-body h1, .md-body h2, .md-body h3,
  .md-body h4, .md-body h5, .md-body h6 {
    font-family: 'Merriweather', Georgia, 'Times New Roman', serif;
    color: var(--heading); line-height: 1.3; scroll-margin-top: 80px;
  }
  .md-body h1 {
    font-size: 2.25rem; font-weight: 800; margin-bottom: 14px;
    background: linear-gradient(135deg, var(--heading) 40%, var(--accent));
    -webkit-background-clip: text; -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .md-body h2 {
    font-size: 1.52rem; margin-top: 52px; margin-bottom: 16px;
    padding-bottom: 10px; border-bottom: 1px solid var(--border);
  }
  .md-body h3 { font-size: 1.26rem; margin-top: 36px; margin-bottom: 12px; }
  .md-body h4 {
    font-size: 1.06rem; margin-top: 28px; margin-bottom: 10px;
    color: var(--accent2); font-family: 'Inter', sans-serif; font-weight: 600;
  }
  .md-body h5 { font-size: 0.94rem; margin-top: 22px; margin-bottom: 8px; color: var(--text-secondary); }
  .md-body h6 {
    font-size: 0.84rem; margin-top: 18px; margin-bottom: 8px;
    color: var(--muted); text-transform: uppercase; letter-spacing: 0.06em;
  }
  .md-body p { margin-bottom: 18px; }
  .md-body a {
    color: var(--accent); text-decoration: none;
    border-bottom: 1px solid rgba(94,234,212,0.28);
    transition: border-color var(--t);
  }
  .md-body a:hover { border-bottom-color: var(--accent); }
  .md-body ul, .md-body ol { margin: 0 0 18px 0; padding-left: 24px; }
  .md-body li { margin-bottom: 5px; }
  .md-body li::marker { color: var(--accent); }
  .md-body blockquote {
    margin: 24px 0; padding: 14px 20px;
    border-left: 3px solid var(--accent); background: var(--accent-glow);
    border-radius: 0 var(--radius) var(--radius) 0;
    color: var(--text-secondary); font-style: italic;
  }
  .md-body code {
    font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
    font-size: 0.84em; background: var(--code-bg); color: var(--accent2);
    padding: 2px 6px; border-radius: 4px; border: 1px solid var(--border);
  }
  .md-body pre {
    background: var(--code-bg); border: 1px solid var(--border);
    border-radius: var(--radius); padding: 20px 24px;
    overflow-x: auto; margin: 24px 0; position: relative;
  }
  .md-body pre code {
    background: none; border: none; padding: 0;
    font-size: 0.84rem; color: var(--text); line-height: 1.78;
  }
  .md-body table {
    width: 100%; border-collapse: collapse;
    margin: 24px 0; font-size: 0.88rem; overflow-x: auto; display: block;
  }
  .md-body th {
    background: var(--surface2); color: var(--accent);
    font-weight: 600; font-size: 0.72rem; letter-spacing: 0.08em;
    text-transform: uppercase; padding: 11px 14px;
    text-align: left; border: 1px solid var(--border);
  }
  .md-body td { padding: 9px 14px; border: 1px solid var(--border); }
  .md-body tr:hover td { background: rgba(255,255,255,0.018); }
  .md-body img {
    max-width: 100%; border-radius: var(--radius);
    border: 1px solid var(--border); margin: 20px 0;
    box-shadow: 0 4px 24px rgba(0,0,0,0.28);
  }
  .md-body hr { border: none; border-top: 1px solid var(--border); margin: 40px 0; }

  /* ── Mermaid container ── */
  .md-body .mermaid {
    background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--radius); padding: 24px; margin: 24px 0;
    overflow-x: auto; text-align: center;
    cursor: pointer; transition: border-color var(--t);
  }
  .md-body .mermaid:hover { border-color: var(--muted); }
  .md-body .mermaid svg { max-width: 100%; height: auto; }

  /* ── Mermaid theme overrides ── */
  .md-body .mermaid .node rect,
  .md-body .mermaid .node circle,
  .md-body .mermaid .node polygon,
  .md-body .mermaid .node path { fill: var(--surface2)!important; stroke: var(--accent)!important; stroke-width: 1.5px!important; }
  .md-body .mermaid .edgeLabel { background: var(--surface)!important; color: var(--text-secondary)!important; }
  .md-body .mermaid .cluster rect { fill: var(--surface)!important; stroke: var(--border)!important; }
  .md-body .mermaid .flowchart-link,
  .md-body .mermaid .messageLine0,
  .md-body .mermaid .messageLine1 { stroke: var(--accent)!important; }
  .md-body .mermaid .arrowheadPath,
  .md-body .mermaid .arrowMarkerPath { fill: var(--accent)!important; stroke: var(--accent)!important; }
  .md-body .mermaid .actor { stroke: var(--accent)!important; fill: var(--surface2)!important; }
  .md-body text.actor { fill: var(--text)!important; stroke: none!important; }
  .md-body .mermaid .sectionTitle { fill: var(--accent2)!important; }
  .md-body .mermaid .today { stroke: var(--accent2)!important; stroke-width: 2px!important; }

  /* ── Footer ── */
  .md-footer {
    margin-top: 64px; padding-top: 24px; border-top: 1px solid var(--border);
    display: flex; justify-content: space-between; align-items: center;
    font-size: 0.72rem; color: var(--muted); flex-wrap: wrap; gap: 8px;
  }
  .md-footer span { font-family: 'JetBrains Mono', monospace; }

  /* ── Sidebar overlay (mobile) ── */
  .sidebar-overlay {
    display: none; position: fixed; inset: 0;
    background: rgba(0,0,0,0.55); z-index: 99;
    backdrop-filter: blur(2px); opacity: 0; transition: opacity var(--t);
  }
  .sidebar-overlay.show { display: block; opacity: 1; }

  /* ── Fullscreen mode ── */
  .app.fullscreen .sidebar { display: none; }
  .app.fullscreen .topbar-toggle { display: none; }

  /* ── Loading overlay ── */
  #loading-overlay {
    position: fixed; inset: 0; z-index: 9999; background: var(--bg);
    display: flex; flex-direction: column;
    align-items: center; justify-content: center; gap: 22px;
    transition: opacity 0.4s ease, visibility 0.4s ease;
  }
  #loading-overlay.hidden { opacity: 0; visibility: hidden; pointer-events: none; }
  .loading-spinner {
    width: 44px; height: 44px; border: 2.5px solid var(--border);
    border-top-color: var(--accent); border-radius: 50%;
    animation: spin 0.75s linear infinite;
  }
  .loading-text { font-size: 0.82rem; color: var(--text-secondary); letter-spacing: 0.05em; }
  .loading-bar { width: 180px; height: 2px; background: var(--border); border-radius: 99px; overflow: hidden; }
  .loading-bar-fill {
    height: 100%; width: 0%;
    background: linear-gradient(90deg, var(--accent), var(--accent2));
    border-radius: 99px;
    animation: loading-progress 1.8s ease-in-out forwards;
  }

  /* ── Mermaid zoom overlay ── */
  .mermaid-zoom-overlay {
    position: fixed; inset: 0; z-index: 99999; background: #000;
    display: none; align-items: center; justify-content: center;
    opacity: 0; transition: opacity 0.18s ease;
  }
  .mermaid-zoom-overlay.open { display: flex; opacity: 1; }
  .mermaid-zoom-body {
    width: 100vw; height: 100vh;
    display: flex; align-items: center; justify-content: center;
    overflow: hidden; padding: 48px; cursor: grab;
  }
  .mermaid-zoom-body.dragging { cursor: grabbing; }
  .mermaid-zoom-body svg { max-width: none; max-height: none; transition: transform 0.1s ease; }
  .mermaid-zoom-close {
    position: fixed; top: 14px; right: 18px; z-index: 100000;
    background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.12);
    color: #fff; cursor: pointer; font-size: 20px; padding: 6px 11px;
    line-height: 1; border-radius: 8px; opacity: 0.7;
    transition: opacity 0.18s, background 0.18s; font-family: system-ui, sans-serif;
  }
  .mermaid-zoom-close:hover { opacity: 1; background: rgba(255,255,255,0.12); }

  /* ── Hint tooltip on mermaid ── */
  .md-body .mermaid::after {
    content: 'Click to zoom';
    display: block; margin-top: 8px;
    font-size: 0.65rem; color: var(--muted);
    font-family: 'Inter', sans-serif; letter-spacing: 0.04em;
  }

  /* ── Responsive ── */
  @media (max-width: 768px) {
    .sidebar {
      position: fixed; top: 0; left: 0; bottom: 0;
      width: var(--sidebar-w)!important; min-width: var(--sidebar-w)!important;
      transform: translateX(-100%); transition: transform var(--t);
      box-shadow: 4px 0 32px rgba(0,0,0,0.5);
    }
    .sidebar.open { transform: translateX(0); }
    .content-wrap { padding: 24px 16px 48px; }
    .md-body h1 { font-size: 1.75rem; }
    .md-body h2 { font-size: 1.35rem; }
    .topbar-badge { display: none; }
    .btn-icon span { display: none; }
    .btn-icon { padding: 7px; }
  }
  @media (max-width: 480px) {
    .topbar { padding: 0 10px; gap: 7px; }
    .content-wrap { padding: 18px 12px 36px; }
    .md-body h1 { font-size: 1.5rem; }
    .md-body pre { padding: 14px; }
    .md-body table { font-size: 0.78rem; }
  }
  @media print {
    .sidebar, .topbar, .progress-bar, .sidebar-overlay,
    #loading-overlay, .mermaid-zoom-overlay { display: none!important; }
    .content-wrap { padding: 0; overflow: visible; }
    body { background: #fff; color: #000; }
    .md-body { max-width: none; }
  }

  /* ── Animations ── */
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes fadeIn { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: translateY(0); } }
  @keyframes loading-progress { 0%{width:0%} 60%{width:70%} 100%{width:92%} }
</style>
<script src="__MERMAID_SRC__"></script>
</head>
<body>

<!-- Loading overlay -->
<div id="loading-overlay">
  <div class="loading-spinner"></div>
  <div class="loading-text">Rendering markdown…</div>
  <div class="loading-bar"><div class="loading-bar-fill"></div></div>
</div>

<!-- App shell -->
<div class="app" id="app">

  <!-- Sidebar -->
  <aside class="sidebar" id="sidebar">
    <div class="sidebar-header">
      <div class="sidebar-logo">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
        MD Reader
      </div>
    </div>
    <div class="sidebar-file">
      <div class="sidebar-file-label">File</div>
      <div class="sidebar-file-name">__FILENAME__</div>
    </div>
    <div class="toc-header">Contents</div>
    <nav class="toc" id="toc"></nav>
    <div class="sidebar-footer">v__VERSION__</div>
  </aside>

  <!-- Sidebar overlay for mobile -->
  <div class="sidebar-overlay" id="overlay" onclick="closeSidebar()"></div>

  <!-- Main content -->
  <div class="main">
    <div class="topbar">
      <button class="topbar-toggle" onclick="toggleSidebar()" title="Toggle sidebar ([ key)">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="6" x2="21" y2="6"/>
          <line x1="3" y1="12" x2="21" y2="12"/>
          <line x1="3" y1="18" x2="21" y2="18"/>
        </svg>
      </button>
      <div class="topbar-title">__FILENAME__</div>
      <span class="topbar-badge" id="word-count">__WORD_COUNT__ words · __READ_TIME__ min read</span>
      <div class="topbar-actions">
        <button class="btn-icon" onclick="toggleFullscreen()" title="Fullscreen (F)">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 3 21 3 21 9"/>
            <polyline points="9 21 3 21 3 15"/>
            <line x1="21" y1="3" x2="14" y2="10"/>
            <line x1="3" y1="21" x2="10" y2="14"/>
          </svg>
          <span>Full</span>
        </button>
        <button class="btn-icon" onclick="window.print()" title="Print">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 6 2 18 2 18 9"/>
            <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/>
            <rect x="6" y="14" width="12" height="8"/>
          </svg>
          <span>Print</span>
        </button>
      </div>
    </div>

    <div class="progress-bar">
      <div class="progress-fill" id="progress"></div>
    </div>

    <div class="content-wrap" id="content-wrap" onscroll="onScroll()">
      <article class="md-body" id="md-body">
        __CONTENT__
      </article>
      <div class="md-footer">
        <span>__FILENAME__</span>
        <span>__WORD_COUNT__ words</span>
      </div>
    </div>
  </div>
</div>

<!-- Mermaid fullscreen zoom overlay -->
<div id="mermaid-zoom-overlay" class="mermaid-zoom-overlay">
  <button class="mermaid-zoom-close" onclick="closeMermaidZoom()">✕</button>
  <div class="mermaid-zoom-body" id="zoom-body"></div>
</div>

<script>
// ── TOC builder ──────────────────────────────────────────────────────────────
function buildTOC() {
  const headings = document.querySelectorAll('.md-body h1,.md-body h2,.md-body h3,.md-body h4');
  const toc = document.getElementById('toc');
  let idx = 0;
  headings.forEach(h => {
    if (!h.id) h.id = 'h-' + (idx++);
    const a = document.createElement('a');
    a.href = '#' + h.id;
    a.textContent = h.textContent;
    a.className = h.tagName.toLowerCase();
    a.addEventListener('click', e => {
      e.preventDefault();
      h.scrollIntoView({ behavior: 'smooth' });
      if (window.innerWidth <= 768) closeSidebar();
    });
    toc.appendChild(a);
  });
}

function updateActiveTOC() {
  const headings = document.querySelectorAll('.md-body h1,.md-body h2,.md-body h3,.md-body h4');
  const links = document.querySelectorAll('.toc a');
  let current = '';
  headings.forEach(h => { if (h.getBoundingClientRect().top - 96 < 0) current = h.id; });
  links.forEach(a => a.classList.toggle('active', a.getAttribute('href') === '#' + current));
}

// ── Scroll progress ──────────────────────────────────────────────────────────
function onScroll() {
  const wrap = document.getElementById('content-wrap');
  const pct = wrap.scrollTop / Math.max(1, wrap.scrollHeight - wrap.clientHeight) * 100;
  document.getElementById('progress').style.width = Math.min(pct, 100) + '%';
  updateActiveTOC();
}

// ── Sidebar ──────────────────────────────────────────────────────────────────
function toggleSidebar() {
  const sb = document.getElementById('sidebar');
  const ov = document.getElementById('overlay');
  if (window.innerWidth <= 768) {
    sb.classList.toggle('open');
    ov.classList.toggle('show');
  } else {
    sb.classList.toggle('collapsed');
  }
}
function closeSidebar() {
  document.getElementById('sidebar').classList.remove('open', 'collapsed');
  document.getElementById('overlay').classList.remove('show');
}

// ── Fullscreen ───────────────────────────────────────────────────────────────
function toggleFullscreen() {
  document.getElementById('app').classList.toggle('fullscreen');
}

// ── Keyboard shortcuts ───────────────────────────────────────────────────────
document.addEventListener('keydown', e => {
  if (e.ctrlKey || e.metaKey || e.altKey) return;
  if (e.key === '[')       { e.preventDefault(); toggleSidebar(); }
  else if (e.key === 'f')  { e.preventDefault(); toggleFullscreen(); }
  else if (e.key === 'Escape') {
    closeMermaidZoom();
    document.getElementById('sidebar').classList.remove('open');
    document.getElementById('overlay').classList.remove('show');
    document.getElementById('app').classList.remove('fullscreen');
  }
});

// ── Mermaid zoom (fullscreen, scroll-zoom, drag-pan) ─────────────────────────
let zoomScale = 1, isDragging = false;
let dragStartX = 0, dragStartY = 0;
let translateX = 0, translateY = 0;

document.addEventListener('click', e => {
  const el = e.target.closest('.mermaid');
  if (el) {
    const svg = el.querySelector('svg');
    if (svg) showMermaidZoom(svg);
  }
});

function showMermaidZoom(svg) {
  const overlay = document.getElementById('mermaid-zoom-overlay');
  const body = document.getElementById('zoom-body');
  body.innerHTML = '';
  const clone = svg.cloneNode(true);
  clone.removeAttribute('width');
  clone.removeAttribute('height');
  clone.style.maxWidth = 'none';
  clone.style.maxHeight = 'none';
  body.appendChild(clone);

  const vw = window.innerWidth - 96, vh = window.innerHeight - 96;
  let nw = vw, nh = vh;
  const vb = clone.getAttribute('viewBox');
  if (vb) {
    const p = vb.split(/[\s,]+/).map(Number);
    if (p.length >= 4) { nw = p[2] || vw; nh = p[3] || vh; }
  }
  zoomScale = Math.min(vw / nw, vh / nh, 2);
  if (zoomScale < 0.1) zoomScale = 0.1;
  translateX = 0; translateY = 0;
  applyZoom(clone);
  overlay.classList.add('open');
}

function applyZoom(svg) {
  if (!svg) svg = document.querySelector('#zoom-body svg');
  if (!svg) return;
  svg.style.transformOrigin = '0 0';
  svg.style.transform = `translate(${translateX}px,${translateY}px) scale(${zoomScale})`;
}

function closeMermaidZoom() {
  document.getElementById('mermaid-zoom-overlay').classList.remove('open');
}

// Scroll-to-zoom (cursor-focused affine transform)
document.addEventListener('wheel', e => {
  const overlay = document.getElementById('mermaid-zoom-overlay');
  if (!overlay.classList.contains('open')) return;
  e.preventDefault();
  const svg = document.querySelector('#zoom-body svg');
  if (!svg) return;
  const rect = svg.getBoundingClientRect();
  const mx = e.clientX - rect.left, my = e.clientY - rect.top;
  const old = zoomScale;
  zoomScale = e.deltaY < 0
    ? Math.min(zoomScale * 1.18, 12)
    : Math.max(zoomScale / 1.18, 0.05);
  const r = zoomScale / old;
  translateX = mx - r * (mx - translateX);
  translateY = my - r * (my - translateY);
  applyZoom(svg);
}, { passive: false });

// Drag-to-pan
document.addEventListener('mousedown', e => {
  const overlay = document.getElementById('mermaid-zoom-overlay');
  if (!overlay.classList.contains('open')) return;
  if (e.target.closest('.mermaid-zoom-close')) return;
  isDragging = true;
  dragStartX = e.clientX - translateX;
  dragStartY = e.clientY - translateY;
  document.getElementById('zoom-body').classList.add('dragging');
});
document.addEventListener('mousemove', e => {
  if (!isDragging) return;
  translateX = e.clientX - dragStartX;
  translateY = e.clientY - dragStartY;
  applyZoom(null);
});
document.addEventListener('mouseup', () => {
  if (isDragging) {
    isDragging = false;
    document.getElementById('zoom-body').classList.remove('dragging');
  }
});

// ── Init ──────────────────────────────────────────────────────────────────────
buildTOC();
onScroll();

mermaid.initialize({
  startOnLoad: false,
  theme: 'dark',
  themeVariables: {
    primaryColor: '#1a1d26',
    primaryTextColor: '#e4e6ed',
    primaryBorderColor: '#5eead4',
    lineColor: '#5eead4',
    secondaryColor: '#0a0b0f',
    tertiaryColor: '#12141a',
    fontFamily: 'Inter, -apple-system, sans-serif',
    fontSize: '14px',
  },
  flowchart: { curve: 'basis', padding: 14 },
  sequence: { actorMargin: 48, messageMargin: 18 },
  gantt: { sectionTitleColor: '#fbbf24' },
});

mermaid.run({ querySelector: '.mermaid' })
  .catch(err => console.warn('Mermaid:', err))
  .finally(() => {
    setTimeout(() => document.getElementById('loading-overlay').classList.add('hidden'), 250);
  });
</script>
</body>
</html>"#;
