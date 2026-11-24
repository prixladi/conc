// @ts-check
(function () {
  // @ts-ignore acquireVsCodeApi is injected by VS Code
  const vscode = acquireVsCodeApi();
  const app = /** @type {HTMLElement} */ (document.getElementById('app'));

  /** @type {any} */
  let state = null;
  /** @type {{project: string, service: string} | null} */
  let selected = null;
  let filter = '';
  let follow = true;

  const ICONS = {
    play: '<svg viewBox="0 0 16 16"><path d="M4 2.5v11l9-5.5z"/></svg>',
    stop: '<svg viewBox="0 0 16 16"><rect x="3.5" y="3.5" width="9" height="9" rx="1"/></svg>',
    restart:
      '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M13 8a5 5 0 1 1-1.6-3.7"/><path d="M13 2.5v2.8h-2.8" stroke-linejoin="round"/></svg>',
    log: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2.5" y="2" width="11" height="12" rx="1.5"/><path d="M5 5.5h6M5 8h6M5 10.5h4"/></svg>',
    more: '<svg viewBox="0 0 16 16"><circle cx="3.5" cy="8" r="1.3"/><circle cx="8" cy="8" r="1.3"/><circle cx="12.5" cy="8" r="1.3"/></svg>',
    file: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 1.8h5l3 3V14H4z"/><path d="M9 1.8V5h3"/></svg>',
    term: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1.8" y="2.5" width="12.4" height="11" rx="1.5"/><path d="M4.5 6l2 2-2 2M8 10.5h3.5"/></svg>',
    clear:
      '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.7 9h5.6l.7-9"/></svg>',
    close: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M4 4l8 8M12 4l-8 8"/></svg>',
    add: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M8 3v10M3 8h10"/></svg>',
    refresh:
      '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M13 8a5 5 0 1 1-1.6-3.7"/><path d="M13 2.5v2.8h-2.8" stroke-linejoin="round"/></svg>',
  };

  const STATUS_LABEL = { RUNNING: 'Running', STOPPED: 'Stopped', EXITED: 'Exited', IDLE: 'Idle' };

  /** @param {string} s */
  function esc(s) {
    return String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] || c);
  }

  /** @param {number} sec */
  function duration(sec) {
    sec = Math.max(0, Math.floor(sec));
    const d = Math.floor(sec / 86400), h = Math.floor((sec % 86400) / 3600), m = Math.floor((sec % 3600) / 60), s = sec % 60;
    if (d) return `${d}d ${h}h`;
    if (h) return `${h}h ${m}m`;
    if (m) return `${m}m ${s}s`;
    return `${s}s`;
  }

  const now = () => Math.floor(Date.now() / 1000);

  /** @param {any} s */
  function serviceTime(s) {
    if (s.status === 'RUNNING' && s.startTime) return `up ${duration(now() - s.startTime)}`;
    if (s.status === 'STOPPED' && s.stopTime) return `stopped ${duration(now() - s.stopTime)} ago`;
    if (s.status === 'EXITED') return s.startTime ? `exited · started ${duration(now() - s.startTime)} ago` : 'exited';
    return s.status === 'IDLE' ? 'never started' : '';
  }

  /**
   * @param {string} command
   * @param {string} project
   * @param {string} [service]
   */
  function send(command, project, service) {
    vscode.postMessage({ type: 'command', command, project, service });
  }

  /** @param {string} icon @param {string} title @param {string} attrs */
  function btn(icon, title, attrs, extraClass = '') {
    return `<button class="icon-btn ${extraClass}" title="${esc(title)}" aria-label="${esc(title)}" ${attrs}>${ICONS[icon]}</button>`;
  }

  // ---------- rendering ----------

  function render() {
    if (!state) {
      app.innerHTML = '<div class="empty">Connecting to daemon…</div>';
      return;
    }

    // Only build the skeleton once so the log pane keeps its scroll position.
    if (!app.querySelector('.layout')) {
      app.innerHTML = `
        <header class="top">
          <div class="brand">
            <h1>Conc</h1>
            <span class="pill" id="daemon-pill"></span>
          </div>
          <div class="toolbar">
            <input id="filter" type="search" placeholder="Filter projects & services" aria-label="Filter">
            <button class="btn" data-global="conc.upsertProject">${ICONS.add}<span>Upsert project</span></button>
            ${btn('refresh', 'Refresh', 'data-global="conc.refresh"')}
          </div>
        </header>
        <section class="tiles" id="tiles"></section>
        <div class="layout">
          <main id="projects"></main>
          <aside id="logs" class="logs hidden"></aside>
        </div>`;
      const input = /** @type {HTMLInputElement} */ (app.querySelector('#filter'));
      input.value = filter;
      input.addEventListener('input', () => {
        filter = input.value.trim().toLowerCase();
        renderProjects();
      });
    }

    const pill = /** @type {HTMLElement} */ (app.querySelector('#daemon-pill'));
    const online = state.snapshot.state === 'online';
    pill.className = `pill ${online ? 'ok' : 'bad'}`;
    pill.textContent = online ? 'daemon online' : 'daemon offline';
    pill.title = state.socketPath;

    renderTiles();
    renderProjects();
    renderLogFrame();
  }

  function renderTiles() {
    const el = /** @type {HTMLElement} */ (app.querySelector('#tiles'));
    const projects = state.snapshot.projects;
    const all = projects.flatMap((/** @type {any} */ p) => p.services);
    const count = (/** @type {string} */ st) => all.filter((/** @type {any} */ s) => s.status === st).length;
    const running = count('RUNNING');
    const total = all.length || 1;
    const seg = (/** @type {string} */ st) => `<span class="seg ${st}" style="flex-grow:${count(st)}"></span>`;

    el.innerHTML = `
      <div class="tile"><div class="value">${projects.length}</div><div class="label">Projects</div></div>
      <div class="tile"><div class="value RUNNING-text">${running}</div><div class="label">Running</div></div>
      <div class="tile"><div class="value">${count('STOPPED') + count('IDLE')}</div><div class="label">Stopped / idle</div></div>
      <div class="tile"><div class="value ${count('EXITED') ? 'EXITED-text' : ''}">${count('EXITED')}</div><div class="label">Exited</div></div>
      <div class="tile wide">
        <div class="label">Fleet health · ${Math.round((running / total) * 100)}% running · restart: ${esc(state.restartBehavior)}</div>
        <div class="bar big">${all.length ? seg('RUNNING') + seg('EXITED') + seg('STOPPED') + seg('IDLE') : '<span class="seg IDLE" style="flex-grow:1"></span>'}</div>
      </div>`;
  }

  function renderProjects() {
    const el = /** @type {HTMLElement} */ (app.querySelector('#projects'));
    if (!el) return;
    const snap = state.snapshot;

    if (snap.state === 'offline') {
      el.innerHTML = `<div class="empty">
        <h2>Daemon is not reachable</h2>
        <p>Start it with <code>systemctl --user start concd</code> or check the <code>conc.socketPath</code> setting.</p>
        <p class="muted">${esc(snap.error || '')}</p>
        <button class="btn" data-global="conc.refresh">${ICONS.refresh}<span>Retry</span></button>
      </div>`;
      return;
    }
    if (snap.projects.length === 0) {
      el.innerHTML = `<div class="empty">
        <h2>No projects yet</h2>
        <p>Register a project from a <code>conc.json</code> file.</p>
        <button class="btn" data-global="conc.upsertProject">${ICONS.add}<span>Upsert project</span></button>
      </div>`;
      return;
    }

    const cards = snap.projects
      .map((/** @type {any} */ p) => {
        const nameMatch = !filter || p.name.toLowerCase().includes(filter);
        const services = nameMatch ? p.services : p.services.filter((/** @type {any} */ s) => s.name.toLowerCase().includes(filter));
        if (!services.length) return '';
        return projectCard(p, services);
      })
      .join('');

    // Keep an open "more" menu open across re-renders triggered by state changes.
    const openMenu = el.querySelector('.menu.open')?.getAttribute('data-menu-for');
    el.innerHTML = cards || '<div class="empty"><p>No matches.</p></div>';
    if (openMenu) {
      [...el.querySelectorAll('.menu')].find((m) => m.getAttribute('data-menu-for') === openMenu)?.classList.add('open');
    }
  }

  /** @param {any} p @param {any[]} services */
  function projectCard(p, services) {
    const running = p.services.filter((/** @type {any} */ s) => s.status === 'RUNNING').length;
    const total = p.services.length;
    const health = running === total ? 'all' : running > 0 ? 'some' : 'none';
    const pn = esc(p.name);
    const bar = p.services
      .map((/** @type {any} */ s) => `<span class="seg ${s.status}" style="flex-grow:1" title="${esc(s.name)}: ${STATUS_LABEL[s.status]}"></span>`)
      .join('');

    return `
      <article class="card health-${health}">
        <header>
          <div class="title">
            <h2>${pn}</h2>
            <span class="count">${running}/${total} running</span>
          </div>
          <div class="actions">
            ${running < total ? btn('play', 'Start all services', `data-cmd="conc.startProject" data-project="${pn}"`, 'go') : ''}
            ${running > 0 ? btn('stop', 'Stop all services', `data-cmd="conc.stopProject" data-project="${pn}"`, 'halt') : ''}
            ${btn('restart', `Restart (${state.restartBehavior})`, `data-cmd="conc.restartProject" data-project="${pn}"`)}
            <div class="menu-wrap">
              ${btn('more', 'More actions', `data-menu="${pn}"`)}
              <div class="menu" data-menu-for="${pn}">
                <button data-cmd="conc.restartProjectHot" data-project="${pn}">Restart hot <span class="muted">running only</span></button>
                <button data-cmd="conc.restartProjectCold" data-project="${pn}">Restart cold <span class="muted">all services</span></button>
                <button data-cmd="conc.showProjectSettings" data-project="${pn}">Show settings</button>
                <hr>
                <button data-cmd="conc.clearProjectLogs" data-project="${pn}">Clear logs</button>
                <button class="danger" data-cmd="conc.removeProject" data-project="${pn}">Remove project…</button>
              </div>
            </div>
          </div>
        </header>
        <div class="bar">${bar}</div>
        <ul class="services">
          ${services.map((s) => serviceRow(p.name, s)).join('')}
        </ul>
      </article>`;
  }

  /** @param {string} project @param {any} s */
  function serviceRow(project, s) {
    const pn = esc(project), sn = esc(s.name);
    const isSel = selected && selected.project === project && selected.service === s.name;
    const attrs = `data-project="${pn}" data-service="${sn}"`;
    return `
      <li class="service ${isSel ? 'selected' : ''}" data-select ${attrs} tabindex="0">
        <span class="dot ${s.status}" title="${STATUS_LABEL[s.status]}"></span>
        <span class="name">${sn}</span>
        <span class="status ${s.status}-text">${STATUS_LABEL[s.status]}</span>
        <span class="time" data-time='${esc(JSON.stringify({ status: s.status, startTime: s.startTime, stopTime: s.stopTime }))}'>${esc(serviceTime(s))}</span>
        <span class="pid">${s.status === 'RUNNING' && s.pid ? `pid ${s.pid}` : ''}</span>
        <span class="actions">
          ${s.status === 'RUNNING'
            ? btn('stop', 'Stop', `data-cmd="conc.stopService" ${attrs}`, 'halt')
            : btn('play', 'Start', `data-cmd="conc.startService" ${attrs}`, 'go')}
          ${btn('restart', 'Restart', `data-cmd="conc.restartService" ${attrs}`)}
          ${btn('log', 'Show logs', `data-select ${attrs}`)}
        </span>
      </li>`;
  }

  function renderLogFrame() {
    const el = /** @type {HTMLElement} */ (app.querySelector('#logs'));
    el.classList.toggle('hidden', !selected);
    app.querySelector('.layout')?.classList.toggle('with-logs', !!selected);
    if (!selected) {
      el.innerHTML = '';
      return;
    }
    const key = `${selected.project}/${selected.service}`;
    if (el.dataset.key === key) {
      updateLogHeader();
      return;
    }
    el.dataset.key = key;
    const attrs = `data-project="${esc(selected.project)}" data-service="${esc(selected.service)}"`;
    el.innerHTML = `
      <header>
        <div class="title">
          <span class="dot" id="log-dot"></span>
          <h2>${esc(selected.project)} <span class="muted">/</span> ${esc(selected.service)}</h2>
        </div>
        <div class="actions">
          <label class="toggle" title="Auto-scroll to the newest output"><input type="checkbox" id="follow" ${follow ? 'checked' : ''}> Follow</label>
          ${btn('file', 'Open log in editor', `data-cmd="conc.openServiceLog" ${attrs}`)}
          ${btn('term', 'Follow log in terminal', `data-cmd="conc.tailServiceLog" ${attrs}`)}
          ${btn('clear', 'Clear service logs', `data-cmd="conc.clearServiceLogs" ${attrs}`)}
          ${btn('close', 'Close log view', 'data-unselect')}
        </div>
      </header>
      <div class="log-meta muted" id="log-meta"></div>
      <pre class="log" id="log-body" tabindex="0"><span class="muted">Loading…</span></pre>`;
    const followBox = /** @type {HTMLInputElement} */ (el.querySelector('#follow'));
    followBox.addEventListener('change', () => {
      follow = followBox.checked;
      if (follow) scrollLogToEnd();
    });
    const body = /** @type {HTMLElement} */ (el.querySelector('#log-body'));
    body.addEventListener('scroll', () => {
      const atEnd = body.scrollHeight - body.scrollTop - body.clientHeight < 8;
      if (follow !== atEnd) {
        follow = atEnd;
        followBox.checked = atEnd;
      }
    });
    updateLogHeader();
  }

  function findSelectedService() {
    if (!selected || !state) return null;
    const p = state.snapshot.projects.find((/** @type {any} */ p) => p.name === selected?.project);
    return p?.services.find((/** @type {any} */ s) => s.name === selected?.service) || null;
  }

  function updateLogHeader() {
    const s = findSelectedService();
    const dot = app.querySelector('#log-dot');
    if (dot) dot.className = `dot ${s ? s.status : 'IDLE'}`;
  }

  function scrollLogToEnd() {
    const body = app.querySelector('#log-body');
    if (body) body.scrollTop = body.scrollHeight;
  }

  // ---------- ANSI colors ----------

  const ANSI_RE = /\x1b\[([0-9;]*)m/g;
  const STRIP_RE = /\x1b\[[0-9;?]*[ -\/]*[@-~]|\x1b\][^\x07]*\x07|\r/g;

  /** @param {string} text */
  function ansiToHtml(text) {
    let out = '';
    let fg = 0, bold = false, open = false, last = 0;
    const flush = (/** @type {string} */ chunk) => {
      chunk = chunk.replace(STRIP_RE, '');
      if (chunk) out += esc(chunk);
    };
    const reopen = () => {
      if (open) out += '</span>';
      open = false;
      const cls = [fg ? `fg${fg}` : '', bold ? 'b' : ''].filter(Boolean).join(' ');
      if (cls) {
        out += `<span class="${cls}">`;
        open = true;
      }
    };
    text.replace(ANSI_RE, (m, codes, idx) => {
      flush(text.slice(last, idx));
      last = idx + m.length;
      for (const c of String(codes || '0').split(';').map(Number)) {
        if (c === 0) { fg = 0; bold = false; }
        else if (c === 1) bold = true;
        else if (c === 22) bold = false;
        else if (c === 39) fg = 0;
        else if ((c >= 30 && c <= 37) || (c >= 90 && c <= 97)) fg = c;
      }
      reopen();
      return m;
    });
    flush(text.slice(last));
    if (open) out += '</span>';
    return out;
  }

  // ---------- events ----------

  document.addEventListener('click', (e) => {
    const t = /** @type {HTMLElement} */ (e.target);

    const menuBtn = t.closest('[data-menu]');
    document.querySelectorAll('.menu.open').forEach((m) => {
      if (!menuBtn || m.getAttribute('data-menu-for') !== menuBtn.getAttribute('data-menu')) m.classList.remove('open');
    });
    if (menuBtn) {
      const name = menuBtn.getAttribute('data-menu');
      const menu = [...document.querySelectorAll('.menu')].find((m) => m.getAttribute('data-menu-for') === name);
      menu?.classList.toggle('open');
      return;
    }

    const global = t.closest('[data-global]');
    if (global) {
      send(global.getAttribute('data-global') || '', '');
      return;
    }

    const cmd = t.closest('[data-cmd]');
    if (cmd) {
      e.stopPropagation();
      send(cmd.getAttribute('data-cmd') || '', cmd.getAttribute('data-project') || '', cmd.getAttribute('data-service') || undefined);
      return;
    }

    if (t.closest('[data-unselect]')) {
      vscode.postMessage({ type: 'unselect' });
      return;
    }

    const sel = t.closest('[data-select]');
    if (sel) {
      vscode.postMessage({ type: 'select', project: sel.getAttribute('data-project'), service: sel.getAttribute('data-service') });
    }
  });

  document.addEventListener('keydown', (e) => {
    const t = /** @type {HTMLElement} */ (e.target);
    if ((e.key === 'Enter' || e.key === ' ') && t.matches('li[data-select]')) {
      e.preventDefault();
      vscode.postMessage({ type: 'select', project: t.getAttribute('data-project'), service: t.getAttribute('data-service') });
    }
    if (e.key === 'Escape') {
      document.querySelectorAll('.menu.open').forEach((m) => m.classList.remove('open'));
    }
  });

  window.addEventListener('message', (event) => {
    const msg = event.data;
    switch (msg.type) {
      case 'state':
        state = msg;
        selected = msg.selected;
        render();
        break;
      case 'selected':
        selected = msg.selected;
        follow = true;
        if (state) render();
        break;
      case 'log': {
        if (!selected || msg.ref.project !== selected.project || msg.ref.service !== selected.service) break;
        const body = app.querySelector('#log-body');
        const meta = app.querySelector('#log-meta');
        if (!body || !meta) break;
        meta.textContent = msg.file ? `${msg.file}${msg.truncated ? ' · showing tail' : ''}` : '';
        body.innerHTML = msg.error
          ? `<span class="muted">${esc(msg.error)}</span>`
          : msg.text
            ? ansiToHtml(msg.text)
            : '<span class="muted">Log is empty.</span>';
        if (follow) scrollLogToEnd();
        break;
      }
    }
  });

  // Tick relative times without re-rendering everything.
  setInterval(() => {
    document.querySelectorAll('[data-time]').forEach((el) => {
      try {
        el.textContent = serviceTime(JSON.parse(el.getAttribute('data-time') || '{}'));
      } catch {
        /* ignore */
      }
    });
  }, 1000);

  render();
  vscode.postMessage({ type: 'ready' });
})();
