// Galaxy Wars – Spielserver
// Liefert die Browser-Version aus und leitet die Züge zwischen zwei Spielern weiter.
// Das Spiel selbst läuft komplett in den Browsern; der Server kennt nur Räume und Nachrichten.

'use strict';
const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { WebSocketServer } = require('ws');

const PORT = Number(process.env.PORT) || 8080;
const PUBLIC_DIR = path.resolve(process.env.PUBLIC_DIR || path.join(__dirname, '..', 'web'));
const MAX_MSG = 256 * 1024;          // größere Nachrichten werden abgelehnt
const EMPTY_ROOM_MS = 15 * 60e3;     // Raum ohne verbundene Spieler nach 15 min entfernen
const IDLE_ROOM_MS = 12 * 3600e3;    // Raum ohne Aktivität nach 12 h entfernen

// ---------- Einstellungen über Umgebungsvariablen (z. B. in der docker-compose.yml) ----------
// CREATE_PASSWORD  Passwort zum Erstellen von Online-Spielen (leer = jeder darf erstellen)
// NOTIFY_URL       Adresse für Push-Benachrichtigungen, z. B. https://ntfy.sh/mein-geheimes-thema
// NOTIFY_SHOW_IP   „true“: IP-Adresse der Besucher in der Benachrichtigung anzeigen
// MAX_ROOMS        höchstens so viele Online-Spiele gleichzeitig (Standard 20)
// MAX_CONN_PER_IP  höchstens so viele Verbindungen pro Absender (Standard 8)
const CREATE_PASSWORD = String(process.env.CREATE_PASSWORD || '');
const NOTIFY_URL = String(process.env.NOTIFY_URL || '');
const NOTIFY_SHOW_IP = /^(1|true|ja|yes)$/i.test(process.env.NOTIFY_SHOW_IP || '');
const MAX_ROOMS = Number(process.env.MAX_ROOMS) || 20;
const MAX_CONN_PER_IP = Number(process.env.MAX_CONN_PER_IP) || 8;
const VISIT_QUIET_MS = 30 * 60e3;    // pro Besucher höchstens eine Aufruf-Meldung je 30 min

// IP des Besuchers – hinter dem Reverse Proxy der Synology steht sie in X-Forwarded-For / X-Real-IP
function clientIp(req) {
  // Der letzte Eintrag stammt vom eigenen Reverse-Proxy; alles davor kann der Besucher selbst mitschicken
  const f = String(req.headers['x-forwarded-for'] || '').split(',').pop().trim();
  return f || String(req.headers['x-real-ip'] || '') || req.socket.remoteAddress || '?';
}
const who = (ip) => (NOTIFY_SHOW_IP ? ` (${ip})` : '');

// Push-Benachrichtigung (ntfy-kompatibel: einfacher POST mit Text), Fehler werden nur protokolliert
function notify(title, text) {
  console.log(`[${new Date().toISOString()}] ${title}: ${text}`);
  if (!NOTIFY_URL) return;
  fetch(NOTIFY_URL, {
    method: 'POST',
    body: text,
    headers: { Title: title, Tags: 'rocket' },
    signal: AbortSignal.timeout(8000),
  }).catch((e) => console.log('Benachrichtigung fehlgeschlagen:', e.message));
}

// Passwort-Vergleich in konstanter Zeit
function passwordOk(p) {
  if (!CREATE_PASSWORD) return true;
  const a = crypto.createHash('sha256').update(String(p || '')).digest();
  const b = crypto.createHash('sha256').update(CREATE_PASSWORD).digest();
  return crypto.timingSafeEqual(a, b);
}

const lastVisit = new Map();         // IP -> Zeitpunkt der letzten Aufruf-Meldung

const MIME = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.png': 'image/png', '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon', '.json': 'application/json',
};

// ---------- Statische Dateien ----------
const server = http.createServer((req, res) => {
  const url = new URL(req.url, 'http://x');
  if (url.pathname === '/health') {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ ok: true, rooms: rooms.size }));
    return;
  }
  if (url.pathname === '/config') { // sagt dem Spiel, ob zum Erstellen ein Passwort nötig ist
    res.writeHead(200, { 'content-type': 'application/json', 'cache-control': 'no-cache' });
    res.end(JSON.stringify({ needPassword: !!CREATE_PASSWORD }));
    return;
  }
  let p;
  try { p = decodeURIComponent(url.pathname); } catch { res.writeHead(400); res.end(); return; }
  if (p.endsWith('/')) p += 'index.html';
  if (p === '/index.html') { // Seitenaufruf melden (gedrosselt pro Besucher)
    const ip = clientIp(req), now = Date.now();
    if (now - (lastVisit.get(ip) || 0) > VISIT_QUIET_MS) {
      lastVisit.set(ip, now);
      const join = url.searchParams.get('join');
      notify('Galaxy Wars: Besuch', join ? `Jemand öffnet einen Einladungslink (${join})${who(ip)}` : `Jemand hat das Spiel aufgerufen${who(ip)}`);
    }
  }
  const file = path.resolve(PUBLIC_DIR, '.' + p);
  if (!file.startsWith(PUBLIC_DIR + path.sep)) { res.writeHead(403); res.end(); return; }
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' }); res.end('Nicht gefunden'); return; }
    res.writeHead(200, {
      'content-type': MIME[path.extname(file)] || 'application/octet-stream',
      'cache-control': 'no-cache',
    });
    res.end(data);
  });
});

// ---------- Räume ----------
// Ein Raum hat zwei Plätze: 0 = Ersteller (erzeugt die Karten), 1 = Gast.
// Jeder Platz hat ein geheimes Token, mit dem man nach einem Verbindungsabbruch zurückkehren kann.
const rooms = new Map();
const ALPHABET = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'; // ohne verwechselbare Zeichen

function newCode() {
  for (;;) {
    let c = '';
    for (let i = 0; i < 4; i++) c += ALPHABET[crypto.randomInt(ALPHABET.length)];
    if (!rooms.has(c)) return c;
  }
}
const token = () => crypto.randomBytes(16).toString('hex');
const online = (ws) => !!ws && ws.readyState === 1;

function send(ws, msg) {
  if (online(ws)) ws.send(JSON.stringify(msg));
}
function cleanName(n) {
  return String(n || '').replace(/[\u0000-\u001f]/g, '').trim().slice(0, 20);
}
function attach(ws, room, seat) {
  room.seats[seat] = ws;
  room.touched = Date.now();
  room.emptySince = 0;
  ws.room = room.code;
  ws.seat = seat;
}
function detach(ws) {
  const room = ws.room && rooms.get(ws.room);
  ws.room = null;
  if (!room || room.seats[ws.seat] !== ws) return null;
  room.seats[ws.seat] = null;
  if (!room.seats[0] && !room.seats[1]) room.emptySince = Date.now();
  return room;
}
function closeRoom(room, reason) {
  for (const s of room.seats) {
    if (s) { send(s, { t: 'closed', reason }); s.room = null; }
  }
  rooms.delete(room.code);
}

const wss = new WebSocketServer({ server, path: '/ws', maxPayload: MAX_MSG });

const connPerIp = new Map();          // IP -> Anzahl offener Verbindungen
const badPw = new Map();              // IP -> { n, since } falsche Passwort-Versuche

wss.on('connection', (ws, req) => {
  // Obergrenze: Verbindungen pro Absender
  const ip = clientIp(req);
  const n = (connPerIp.get(ip) || 0) + 1;
  if (n > MAX_CONN_PER_IP) { ws.close(1008, 'Zu viele Verbindungen'); return; }
  connPerIp.set(ip, n);
  ws.ip = ip;
  ws.alive = true;
  ws.msgs = 0;
  ws.on('pong', () => { ws.alive = true; });

  ws.on('message', (raw) => {
    // Drosselung: höchstens 60 Nachrichten pro Sekunde, sonst wird ignoriert bzw. getrennt
    if (++ws.msgs > 60) { if (ws.msgs > 300) ws.terminate(); return; }
    let m;
    try { m = JSON.parse(raw); } catch { return; }
    if (!m || typeof m.t !== 'string') return;

    switch (m.t) {
      case 'create': {
        // nach 5 Fehlversuchen 10 Minuten Sperre für diesen Absender
        const b = badPw.get(ip);
        if (b && b.n >= 5 && Date.now() - b.since < 10 * 60e3) {
          send(ws, { t: 'error', reason: 'Zu viele falsche Versuche – bitte später noch einmal.', needPassword: true });
          return;
        }
        if (!passwordOk(m.password)) {
          const e = b && Date.now() - b.since < 10 * 60e3 ? b : { n: 0, since: Date.now() };
          e.n++; badPw.set(ip, e);
          if (e.n === 1 || e.n === 5) notify('Galaxy Wars: falsches Passwort', `Jemand${who(ip)} wollte ohne gültiges Passwort ein Online-Spiel erstellen` + (e.n === 5 ? ' (5 Versuche, jetzt gesperrt)' : ''));
          send(ws, { t: 'error', reason: m.password ? 'Das Passwort stimmt nicht.' : 'Zum Erstellen eines Online-Spiels ist ein Passwort nötig.', needPassword: true });
          return;
        }
        badPw.delete(ip);
        if (rooms.size >= MAX_ROOMS) { send(ws, { t: 'error', reason: 'Der Server ist gerade voll. Bitte später noch einmal.' }); return; }
        const old = detach(ws);
        if (old && ws.seat === 0) closeRoom(old, 'host-left');
        const code = newCode();
        const room = { code, seats: [null, null], tokens: [token(), null], names: [cleanName(m.name), ''], touched: 0, emptySince: 0 };
        rooms.set(code, room);
        attach(ws, room, 0);
        send(ws, { t: 'created', room: code, seat: 0, token: room.tokens[0], names: room.names });
        notify('Galaxy Wars: neues Online-Spiel', `${room.names[0] || 'Jemand'}${who(ip)} hat das Spiel ${code} erstellt`);
        return;
      }
      case 'join': {
        const room = rooms.get(String(m.room || '').toUpperCase().trim());
        if (!room) { send(ws, { t: 'error', reason: 'Kein Spiel mit diesem Code gefunden.' }); return; }
        if (online(room.seats[1])) { send(ws, { t: 'error', reason: 'Dieses Spiel ist schon voll.' }); return; }
        detach(ws);
        room.tokens[1] = token();
        room.names[1] = cleanName(m.name);
        attach(ws, room, 1);
        send(ws, { t: 'joined', room: room.code, seat: 1, token: room.tokens[1], names: room.names, peer: online(room.seats[0]) });
        send(room.seats[0], { t: 'peer-joined', names: room.names });
        notify('Galaxy Wars: Mitspieler beigetreten', `${room.names[1] || 'Jemand'}${who(ip)} spielt jetzt im Spiel ${room.code} gegen ${room.names[0] || 'den Ersteller'}`);
        return;
      }
      case 'rejoin': {
        const room = rooms.get(String(m.room || '').toUpperCase().trim());
        const seat = m.seat === 1 ? 1 : 0;
        if (!room || !room.tokens[seat] || room.tokens[seat] !== m.token) {
          send(ws, { t: 'error', reason: 'Das Spiel gibt es nicht mehr.', rejoin: true });
          return;
        }
        const prev = room.seats[seat];
        if (prev && prev !== ws) { prev.room = null; prev.terminate(); }
        detach(ws);
        attach(ws, room, seat);
        send(ws, { t: 'rejoined', room: room.code, seat, names: room.names, peer: online(room.seats[1 - seat]) });
        send(room.seats[1 - seat], { t: 'peer-joined', names: room.names });
        return;
      }
      case 'leave': {
        const seat = ws.seat;
        const room = detach(ws);
        if (!room) return;
        if (seat === 0) closeRoom(room, 'host-left');
        else { room.tokens[1] = null; send(room.seats[0], { t: 'peer-left', final: true }); }
        return;
      }
      case 'relay': {
        // Spielnachrichten werden unverändert an den Mitspieler weitergereicht
        const room = ws.room && rooms.get(ws.room);
        if (!room) return;
        room.touched = Date.now();
        send(room.seats[1 - ws.seat], { t: 'relay', data: m.data });
        return;
      }
    }
  });

  ws.on('close', () => {
    const c = (connPerIp.get(ws.ip) || 1) - 1;
    if (c > 0) connPerIp.set(ws.ip, c); else connPerIp.delete(ws.ip);
    const seat = ws.seat;
    const room = detach(ws);
    if (room) send(room.seats[1 - seat], { t: 'peer-left' });
  });
});

// Verbindungen lebendig halten (Proxys schließen sonst stille Verbindungen) und alte Räume aufräumen
// Nachrichten-Zähler jede Sekunde zurücksetzen (Drosselung)
setInterval(() => { for (const ws of wss.clients) ws.msgs = 0; }, 1000).unref();

setInterval(() => {
  const old = Date.now() - 24 * 3600e3; // alte Einträge vergessen
  for (const [k, t] of lastVisit) if (t < old) lastVisit.delete(k);
  for (const [k, b] of badPw) if (b.since < old) badPw.delete(k);
  for (const ws of wss.clients) {
    if (!ws.alive) { ws.terminate(); continue; }
    ws.alive = false;
    ws.ping();
  }
  const now = Date.now();
  for (const room of rooms.values()) {
    if ((room.emptySince && now - room.emptySince > EMPTY_ROOM_MS) || now - room.touched > IDLE_ROOM_MS) {
      closeRoom(room, 'timeout');
    }
  }
}, 25000);

// Sauber beenden, wenn der Container gestoppt wird (Node reagiert als Hauptprozess
// im Container sonst nicht auf SIGTERM, und das Stoppen „schlägt fehl“)
for (const sig of ['SIGTERM', 'SIGINT']) {
  process.on(sig, () => {
    console.log(`${sig} empfangen – Galaxy Wars wird beendet`);
    for (const ws of wss.clients) ws.close(1001, 'Server wird neu gestartet');
    server.close(() => process.exit(0));
    setTimeout(() => process.exit(0), 2000).unref();
  });
}

server.listen(PORT, () => {
  console.log(`Galaxy Wars läuft auf Port ${PORT} (Dateien aus ${PUBLIC_DIR})`);
  console.log(`Passwort zum Erstellen: ${CREATE_PASSWORD ? 'aktiv' : 'aus'} · Benachrichtigungen: ${NOTIFY_URL ? 'aktiv' : 'aus'}`);
});
