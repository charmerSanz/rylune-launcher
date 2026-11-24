import React, { useEffect, useMemo, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  Home,
  Layers,
  FolderOpen,
  Users,
  Settings,
  Terminal,
  Play,
  Plus,
  ArrowUpRight,
  Download,
  Check,
  Minus,
  Square,
  X,
  Search,
  Gamepad2,
  MemoryStick,
  Cpu,
  ShieldCheck,
  RefreshCw,
  Trash2,
  Edit3,
  Copy,
  Info,
  Box,
  ChevronRight,
  Bell,
  UserRound,
  Sparkles,
  Clock3,
  FolderCog,
  MonitorSmartphone,
  Rocket,
  HardDriveDownload,
  MoreHorizontal,
  Grid3X3,
  SlidersHorizontal,
} from 'lucide-react';
import './style.css';

const R = '1.21.11';
const supported = v => /^(1\.21\.11|26\.[123](\.(0|[1-9][0-9]*))?)$/.test(v || '');
const featured = ['1.21.11', '26.2', '26.3'];
const folderName = p => p.id === p.version ? p.version : `${p.version}-${p.id}`;
const defaultConfig = {
  profiles: [{ id: R, name: 'Minecraft 1.21.11', version: R, ram: 2048, java: '' }],
  accounts: [],
  profile: R,
  account: '',
  client_id: '',
};

const nav = [
  ['Home', Home],
  ['Versions', Layers],
  ['Profiles', FolderOpen],
  ['Accounts', Users],
  ['Settings', Settings],
  ['Logs', Terminal],
];

const native = !!window.__TAURI_INTERNALS__;
const call = (cmd, args = {}) =>
  native ? invoke(cmd, args) : Promise.reject(new Error('Open the Tauri desktop application to use this feature.'));
const delay = (ms) => new Promise((r) => setTimeout(r, ms));

function App() {
  const [cfg, setCfg] = useState(defaultConfig);
  const [page, setPage] = useState('Home');
  const [installed, setInstalled] = useState([]);
  const [online, setOnline] = useState([]);
  const [catalog, setCatalog] = useState([]);
  const [search, setSearch] = useState('');
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState('Ready to play');
  const [progress, setProgress] = useState(0);
  const [logs, setLogs] = useState([]);
  const [modal, setModal] = useState(null);
  const [toast, setToast] = useState('');
  const [settingsValue, setSettingsValue] = useState('');
  const [device, setDevice] = useState(null);
  const [installedOnly, setInstalledOnly] = useState(false);

  const cfgRef = useRef(cfg);
  const loginRef = useRef(0);
  const logEnd = useRef(null);

  const selectedProfile = cfg.profiles.find((x) => x.id === cfg.profile) || cfg.profiles[0];
  const selectedAccount = cfg.accounts.find((x) => x.id === cfg.account);

  const profileVersions = useMemo(() => [...new Set([R, '26.1', '26.2', '26.3', ...installed, ...catalog.map(v => v.id), modal?.value?.version].filter(supported))], [installed, catalog, modal]);
  const versionRows = useMemo(() => {
    const rows = catalog.length ? catalog : ['1.21.11','26.1','26.2','26.3'].map(id => ({id, type:'release'}));
    return rows.filter(v => supported(v.id) && v.type === 'release' && (!installedOnly || installed.includes(v.id)));
  }, [catalog, installedOnly, installed]);
  const featureVersions = featured.map(id => ({id, title:id+' Fabric', subtitle:'Minecraft Java Edition'}));

  function notify(e) {
    setToast(String(e?.message || e));
  }

  function refresh() {
    call('installed_versions').then(setInstalled).catch(() => {});
    call('signed_in').then(setOnline).catch(() => {});
  }

  useEffect(() => {
    if (!native) return;
    call('load_config')
      .then((c) => {
        const v = { ...defaultConfig, ...c };
        cfgRef.current = v;
        setCfg(v);
        setSettingsValue(v.client_id || '');
      })
      .catch(notify);
    refresh();

    let stopped = false;
    let un = [];
    Promise.all([
      listen('launcher-log', (e) => {
        setStatus(e.payload);
        setLogs((l) => [...l.slice(-1499), e.payload]);
      }),
      listen('launcher-progress', (e) => setProgress(e.payload)),
    ]).then((x) => {
      if (stopped) x.forEach((f) => f());
      else un = x;
    });

    return () => {
      stopped = true;
      un.forEach((f) => f());
    };
  }, []);

  useEffect(() => {
    if ((page === 'Versions' || page === 'Home') && !catalog.length && native) loadCatalog();
  }, [page]);

  useEffect(() => {
    if (page === 'Logs') logEnd.current?.scrollIntoView({ block: 'end' });
  }, [logs, page]);

  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(''), 7500);
    return () => clearTimeout(t);
  }, [toast]);

  async function save(patch) {
    const next = { ...cfgRef.current, ...patch };
    if (native) await call('save_config', { config: next });
    cfgRef.current = next;
    setCfg(next);
  }

  function action(fn) {
    Promise.resolve().then(fn).catch(notify);
  }

  async function loadCatalog() {
    try {
      const data = await call('catalog');
      setCatalog(data.versions || []);
    } catch (e) {
      notify(e);
    }
  }

  async function run(installOnly = false, version) {
    if (busy) return;
    if (!selectedAccount) {
      setPage('Accounts');
      notify('Select an account first.');
      return;
    }
    let target = selectedProfile;
    if (version) {
      target = selectedProfile?.version === version ? selectedProfile : cfgRef.current.profiles.find(p => p.version === version);
      if (!target) {
        target = {id:version, name:`Minecraft ${version}`, version, ram:2048, java:""};
        if (cfgRef.current.profiles.some(p => p.id === target.id)) target.id = crypto.randomUUID();
        try { await save({profiles:[...cfgRef.current.profiles,target],profile:target.id}); } catch(e){notify(e);return;}
      }
    }
    if (!target) {
      setPage('Profiles');
      notify('Create or select a profile first.');
      return;
    }

    setBusy(true);
    setProgress(0);
    setStatus(installOnly ? 'Preparing installation…' : 'Preparing Minecraft…');

    try {
      await call('run_game', {
        profile: target,
        account: selectedAccount,
        installOnly,
      });
      setStatus(installOnly ? 'Installation complete' : 'Game session ended');
      refresh();
    } catch (e) {
      setStatus('Operation failed');
      notify(e);
      setLogs((l) => [...l, String(e)]);
    } finally {
      setBusy(false);
    }
  }

  async function selectVersion(version) {
    if (busy) return;
    const existing = cfgRef.current.profiles.find(p => p.version === version);
    if (existing) { await save({ profile: existing.id }); return; }
    setModal({ kind: 'profile', value: { id: crypto.randomUUID(), name: `Minecraft ${version}`, version, ram: 2048, java: '' } });
  }

  function edit(profile) {
    setModal({
      kind: 'profile',
      value: profile || { id: crypto.randomUUID(), name: '', version: R, ram: 2048, java: '' },
    });
  }

  async function saveProfile() {
    const v = modal.value;
    if (!v.name.trim() || !supported(v.version) || !Number.isInteger(+v.ram) || +v.ram < 512 || +v.ram > 65536) {
      throw Error('Enter a name, supported Minecraft release (1.21.11–26.3), and valid RAM.');
    }
    const old = cfgRef.current.profiles.find(x => x.id === v.id);
    const rest = cfgRef.current.profiles.filter(x => x.id !== v.id);
    const id = !old && !rest.some(x => x.id === v.version || x.version === v.version) ? v.version : v.id;
    await save({profiles:[...rest,{...v,id,ram:+v.ram}],profile:id});
    setModal(null);
  }

  async function addOffline() {
    const v = await call('offline_account', { name: modal.value.trim() });
    await save({ accounts: [...cfg.accounts.filter((x) => x.id !== v.id), v], account: v.id });
    setModal(null);
  }

  async function login() {
    if (!cfg.client_id) {
      setPage('Settings');
      notify('Enter your approved Microsoft app client ID first.');
      return;
    }

    const attempt = ++loginRef.current;
    setModal({ kind: 'login' });
    setDevice(null);

    try {
      const d = await call('start_login', { clientId: cfg.client_id });
      if (attempt !== loginRef.current) return;
      setDevice(d);
      let interval = d.interval;
      for (;;) {
        await delay(interval * 1000);
        if (attempt !== loginRef.current) return;
        const r = await call('poll_login', { session: d.session });
        if (attempt !== loginRef.current) return;
        if (r.pending) {
          interval = r.interval || interval;
          continue;
        }
        const v = r.account;
        await save({
          accounts: [...cfgRef.current.accounts.filter((x) => x.id !== v.id), v],
          account: v.id,
        });
        refresh();
        setModal(null);
        notify('Microsoft account connected');
        return;
      }
    } catch (e) {
      if (attempt === loginRef.current) {
        setModal(null);
        notify(e);
      }
    }
  }

  function closeModal() {
    loginRef.current++;
    setModal(null);
  }

  const isInstalled = v => installed.includes(v);
  const win = (fn) => action(() => (native ? fn(getCurrentWindow()) : Promise.resolve()));
  const change = (k, v) => setModal((m) => ({ ...m, value: { ...m.value, [k]: v } }));

  const PageIcon = nav.find(([label]) => label === page)?.[1] || Home;

  return (
    <div className="app-shell">
      

      <header className="topbar" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <img src="/logo.png" alt="Rylune" />
          <div>
            <strong>RYLUNE</strong>

          </div>
        </div>

        <nav className="topnav">
          {nav.map(([label, Icon]) => (
            <button key={label} className={`tab ${page === label ? 'active' : ''}`} onClick={() => setPage(label)}>
              <Icon size={16} />
              <span>{label}</span>
            </button>
          ))}
        </nav>

        <div className="topbar-actions">
          <button className="profile-chip" onClick={() => setPage('Accounts')}>
            <div className="avatar skin-avatar">{selectedAccount?.name?.[0]?.toUpperCase() || <UserRound size={18} />}</div>
            <div className="profile-chip-text">
              <strong>{selectedAccount?.name || 'Add account'}</strong>
              <span>{selectedAccount?.type || 'Sign in'}</span>
            </div>
          </button>
          <div className="window-controls">
            <button aria-label="Minimize" onClick={() => win((w) => w.minimize())}><Minus size={17} /></button>
            <button aria-label="Maximize" onClick={() => win((w) => w.toggleMaximize())}><Square size={13} /></button>
            <button aria-label="Close" className="close" onClick={() => win((w) => w.close())}><X size={18} /></button>
          </div>
        </div>
      </header>

      <main className="page-wrap">
        {page === 'Home' && (
          <section className="home-view">
            <div className="hero-panel hero-wallpaper">
              <div className="hero-overlay" />
              <div className="hero-content">
                <div className="hero-copy">
                  <div className="hero-badge"><span className="red-line" />RYLUNE FABRIC</div>
                  <h1>{selectedProfile?.version ? `${selectedProfile.version} Fabric` : 'Minecraft'}</h1>
                  <p>Minecraft Java Edition · Your world. Your rules.</p>
                  <div className="hero-actions">
                    <button className="primary large" disabled={busy} onClick={() => run()}><Play size={18} fill="currentColor" />{busy ? 'Working' : 'Launch game'}</button>
                    <button className="glass-btn configure" title="Configure active profile" onClick={() => edit(selectedProfile)}><SlidersHorizontal size={19}/></button>
                  </div>
                </div>
              </div>
              <span className="hero-caption">MINECRAFT / RYLUNE</span>
            </div>
            <div className="home-bottom-row">
              <div className="section-title-block">
                <h2>Play your way</h2>
                <button className="text-btn" onClick={() => setPage('Versions')}>All versions <ArrowUpRight size={16}/></button>
              </div>
              <div className="featured-grid">
                {featureVersions.map((item, index) => (
                  <button key={`${item.id}-${index}`} className={`featured-card ${selectedProfile?.version === item.id ? 'selected' : ''}`} disabled={busy} onClick={() => action(() => selectVersion(item.id))}>
                    <div className="featured-backdrop" style={{ backgroundImage: `url(${index === 1 ? '/bg-feature.png' : '/bg-card.png'})` }} />
                    <div className="featured-overlay" />
                    {selectedProfile?.version === item.id && <span className="card-check"><Check size={15}/></span>}
                    <div className="featured-info"><h3>{item.title}</h3><p>{item.subtitle}</p></div>
                  </button>
                ))}
                <button className="library-card" onClick={() => setPage('Versions')}><Grid3X3 size={24}/><strong>Library</strong></button>
              </div>
            </div>
          </section>
        )}

        {page !== 'Home' && (
          <div className="page-head">
            <div className="page-title">
              <div className="page-icon"><PageIcon size={18} /></div>
              <div>
                <span className="eyebrow">RYLUNE DESKTOP</span>
                <h1>{page}</h1>
                <p>
                  {page === 'Versions' && 'Install and manage Minecraft versions.'}
                  {page === 'Profiles' && 'Create separate launch setups with custom RAM and Java.'}
                  {page === 'Accounts' && 'Switch between premium and offline accounts.'}
                  {page === 'Settings' && 'Configure launcher-wide preferences and OAuth.'}
                  {page === 'Logs' && 'Read the live launcher and game output.'}
                </p>
              </div>
            </div>

            {page === 'Versions' && (
              <button className="glass-btn" onClick={loadCatalog}><RefreshCw size={16} /> Refresh</button>
            )}
            {page === 'Profiles' && (
              <button className="primary" onClick={() => edit()}><Plus size={16} /> Create profile</button>
            )}
          </div>
        )}

        {page === 'Versions' && (
          <section className="page-panel">
            <div className="toolbar-row">
              <div className="searchbar">
                <Search size={18} />
                <input placeholder="Search versions..." value={search} onChange={(e) => setSearch(e.target.value)} />
              </div>
              <label className="switch-row">
                <input type="checkbox" checked={installedOnly} onChange={(e) => setInstalledOnly(e.target.checked)} />
                <span>Installed only</span>
              </label>
            </div>

            <div className="version-list">
              {versionRows
                .filter((v) => v.id.toLowerCase().includes(search.toLowerCase()))
                .slice(0, 100)
                .map((v) => (
                  <div className="version-row" key={v.id}>
                    <div className="version-row-left">
                      <div className="version-icon">{v.id === R ? <img src="/logo.png" alt="Rylune" /> : <Box size={22} />}</div>
                      <div>
                        <h3>{v.id} Fabric</h3>
                        <p>Minecraft Java Edition · Fabric Loader</p>
                      </div>
                    </div>
                    <div className="version-row-right">
                      {isInstalled(v.id) && <span className="status-pill success"><Check size={14} /> Installed in a profile</span>}
                      <button className="glass-btn" disabled={busy} onClick={() => run(true, v.id)}><Download size={16} /> {isInstalled(v.id) ? 'Repair' : 'Install'}</button>
                      <button className="icon-square" title="Create profile" onClick={() => setModal({ kind: 'profile', value: { id: crypto.randomUUID(), name: v.id, version: v.id, ram: 2048, java: '' } })}><Plus size={18} /></button>
                    </div>
                  </div>
                ))}
            </div>
          </section>
        )}

        {page === 'Profiles' && (
          <section className="page-panel profile-grid">
            {cfg.profiles.map((v) => (
              <article className={`profile-card ${v.id === selectedProfile?.id ? 'selected' : ''}`} key={v.id}>
                <div className="profile-topline">
                  <div className="profile-icon"><Gamepad2 size={22} /></div>
                  {v.id === selectedProfile?.id && <span className="status-pill success"><Check size={14} /> Active</span>}
                </div>
                <h3>{v.name}</h3>
                <p>{v.version} Fabric · profiles/{folderName(v)}</p>
                <div className="metric-row">
                  <span><MemoryStick size={15} /> {v.ram} MB</span>
                  <span><Cpu size={15} /> {v.java ? 'Custom Java' : 'System Java'}</span>
                </div>
                <div className="card-actions">
                  <button className="glass-btn" onClick={() => action(() => save({ profile: v.id }))}>Select</button>
                  <button className="icon-square" onClick={() => edit(v)} title="Edit"><Edit3 size={16} /></button>
                  <button className="icon-square" onClick={() => action(() => call('open_folder', { profile: v.id }))} title="Open full game folder"><FolderOpen size={17} /></button>
                  <button className="icon-square" title="Remove profile" disabled={cfg.profiles.length < 2 || busy} onClick={() => setModal({ kind: 'removeProfile', value: v })}><Trash2 size={16} /></button>
                </div>
              </article>
            ))}
          </section>
        )}

        {page === 'Accounts' && (
          <section className="accounts-view">
            <div className="account-choice-grid">
              <button className="action-panel" onClick={login}>
                <div className="action-icon"><ShieldCheck size={24} /></div>
                <h3>Microsoft account</h3>
                <p>Sign in using device code flow for premium Minecraft Java accounts.</p>
                <span>Connect premium <ArrowUpRight size={14} /></span>
              </button>
              <button className="action-panel" onClick={() => setModal({ kind: 'offline', value: '' })}>
                <div className="action-icon"><UserRound size={24} /></div>
                <h3>Offline account</h3>
                <p>Local play with an existing, legitimately owned installation.</p>
                <span>Add local account <Plus size={14} /></span>
              </button>
            </div>

            <div className="page-subsection">
              <div>
                <span className="eyebrow">Saved identities</span>
                <h2>Accounts</h2>
              </div>
              <span className="muted-caption">{cfg.accounts.length} total</span>
            </div>

            {!cfg.accounts.length && <div className="empty-state">No accounts yet. Add a premium or offline account to continue.</div>}

            <div className="stack-list">
              {cfg.accounts.map((v) => (
                <div className="list-card" key={v.id}>
                  <div className="list-card-left">
                    <div className="avatar large">{v.name[0].toUpperCase()}</div>
                    <div>
                      <h3>{v.name}</h3>
                      <p>{v.type}{v.type === 'Microsoft' && !online.includes(v.id) ? ' · Sign-in required' : ''}</p>
                    </div>
                  </div>
                  <div className="list-card-right">
                    {v.id === selectedAccount?.id && <span className="status-pill success"><Check size={14} /> Active</span>}
                    <button className="glass-btn" onClick={() => action(() => save({ account: v.id }))}>Select</button>
                    <button
                      className="icon-square"
                      title="Remove account"
                      onClick={() =>
                        action(async () => {
                          await call('sign_out', { id: v.id });
                          const list = cfg.accounts.filter((x) => x.id !== v.id);
                          await save({ accounts: list, account: selectedAccount?.id === v.id ? list[0]?.id || '' : cfg.account });
                          refresh();
                        })
                      }
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </section>
        )}

        {page === 'Settings' && (
          <section className="settings-view">
            <div className="info-panel">
              <h3>Microsoft authentication</h3>
              <p>Enter the OAuth application client ID you created for device code sign-in.</p>
              <label>
                Application client ID
                <input value={settingsValue} onChange={(e) => setSettingsValue(e.target.value)} placeholder="Your OAuth application ID" />
              </label>
              <button className="primary" onClick={() => action(async () => { await save({ client_id: settingsValue.trim() }); notify('Settings saved'); })}>
                Save settings
              </button>
            </div>
            <div className="info-panel">
              <h3>Launcher storage</h3>
              <p>Open %APPDATA%/Rylune. Each profile contains its own complete Minecraft game directory.</p>
              <button className="glass-btn" onClick={() => action(() => call('open_folder', { profile: null }))}><FolderOpen size={16} /> Open data directory</button>
            </div>
            <div className="info-panel">
              <h3>About Rylune</h3>
              <p>Version 0.6.0 alpha · Tauri 2 · React · Rust. Modern Fabric edition (1.21.11–26.3).</p>
            </div>
          </section>
        )}

        {page === 'Logs' && (
          <section className="page-panel">
            <div className="toolbar-row toolbar-end">
              <button className="glass-btn" onClick={() => action(() => navigator.clipboard.writeText(logs.join('\n')))}><Copy size={16} /> Copy logs</button>
              <button className="glass-btn" onClick={() => setLogs([])}>Clear</button>
            </div>
            <div className="console-view">
              {logs.length ? logs.map((l, i) => <div key={i}>{l}</div>) : 'Waiting for launcher output…'}
              <div ref={logEnd} />
            </div>
          </section>
        )}
      </main>

      <footer className={`bottom-bar ${page === 'Home' ? 'home-footer' : ''}`}>
        <div className="bottom-left">
          <div className="game-mark"><img src="/logo.png" alt="Rylune" /></div>
          <div className="active-block">
            <label>ACTIVE PROFILE</label>
            <select value={selectedProfile?.id || ''} onChange={(e) => action(() => save({ profile: e.target.value }))}>
              {cfg.profiles.map((v) => <option key={v.id} value={v.id}>{v.name}</option>)}
            </select>
            <span>{selectedProfile?.version} · {selectedProfile?.ram} MB</span>
          </div>
        </div>

        <div className="bottom-center">
          <div className="status-line"><i className={busy ? 'status-dot pulse' : 'status-dot'} /> {status}</div>
          <div className="progress-track"><div style={{ width: `${busy ? Math.max(progress, 5) : 0}%` }} /></div>
        </div>

        <button className="play-button" disabled={busy} onClick={() => run()}>
          <Play size={20} fill="currentColor" />
          <div>
            <strong>{busy ? 'Working' : 'Play'}</strong>
            <span>{busy ? 'Please wait' : 'Launch Minecraft'}</span>
          </div>
        </button>
      </footer>

      {toast && (
        <div className="toast">
          <Info size={17} />
          <span>{toast}</span>
          <button className="icon-square" onClick={() => setToast('')}><X size={15} /></button>
        </div>
      )}

      {modal && (
        <div className="overlay" onMouseDown={(e) => e.target === e.currentTarget && closeModal()}>
          <div className="modal-card">
            <button className="icon-square modal-close" onClick={closeModal}><X size={18} /></button>

            {modal.kind === 'profile' && (
              <>
                <h2>Configure profile</h2>
                <p>Choose the version, RAM, and optional Java executable for this instance.</p>
                <label>
                  Profile name
                  <input autoFocus value={modal.value.name} onChange={(e) => change('name', e.target.value)} />
                </label>
                <label>
                  Minecraft version
                  <input list="version-options" value={modal.value.version} disabled={busy} onChange={(e) => change('version', e.target.value)} />
                  <datalist id="version-options">
                    {profileVersions.map((v) => <option key={v} value={v} />)}
                  </datalist>
                </label>
                <label>
                  Allocated RAM (MB)
                  <input type="number" min="512" max="65536" step="512" value={modal.value.ram} onChange={(e) => change('ram', e.target.value)} />
                </label>
                <label>
                  Java executable
                  <input value={modal.value.java} placeholder="Blank = auto-detect installed Java" onChange={(e) => change('java', e.target.value)} />
                </label>
                <small>1.21.11 uses Java 21; 26.x uses Java 25. The launcher checks official version metadata. Changing the game version uses a different directory; old files are kept.</small>
                <button className="primary full" onClick={() => action(saveProfile)}>Save profile</button>
              </>
            )}

            {modal.kind === 'offline' && (
              <>
                <h2>Add offline account</h2>
                <p>For local play with installed game files. Not valid for authenticated multiplayer.</p>
                <label>
                  Username
                  <input autoFocus maxLength={16} value={modal.value} onChange={(e) => setModal({ ...modal, value: e.target.value })} />
                </label>
                <button className="primary full" onClick={() => action(addOffline)}>Create account</button>
              </>
            )}

            {modal.kind === 'login' && (
              <>
                <h2>Connect Microsoft</h2>
                <p>{device ? 'Open the Microsoft page and enter the code below.' : 'Requesting a device code…'}</p>
                {device && (
                  <>
                    <div className="device-code">{device.code}</div>
                    <p className="device-url">{device.url}</p>
                    <button className="glass-btn" onClick={() => action(() => navigator.clipboard.writeText(device.code))}><Copy size={15} /> Copy code</button>
                    <p className="helper-text">Waiting for Microsoft authentication…</p>
                  </>
                )}
              </>
            )}

            {modal.kind === 'removeProfile' && (
              <>
                <h2>Remove profile?</h2>
                <p>The launcher entry will be removed, but the files on disk will remain untouched.</p>
                <button
                  className="primary full"
                  onClick={() =>
                    action(async () => {
                      const list = cfg.profiles.filter((x) => x.id !== modal.value.id);
                      await save({ profiles: list, profile: selectedProfile?.id === modal.value.id ? list[0].id : cfg.profile });
                      closeModal();
                    })
                  }
                >
                  Remove profile
                </button>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

createRoot(document.getElementById('root')).render(<App />);
