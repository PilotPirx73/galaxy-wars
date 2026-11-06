# Galaxy Wars 2026 auf der Synology hosten

Mit dieser Anleitung läuft Galaxy Wars 2026 als kleiner Server auf deiner Synology. Dein Mitspieler öffnet dann nur noch einen Link im Browser, ohne etwas zu installieren, und ihr spielt online gegeneinander.

Die Anleitung bezieht sich auf **DSM 7.2**. In älteren Versionen heißt der Container Manager noch „Docker“.

## Wie es funktioniert

```
Browser (du)  ─┐                                       ┌─ Container „galaxy-wars“
               ├─ https://deine-adresse:8443 ─ Router ─ Synology-Reverse-Proxy ─┤   (Port 8090)
Browser (Lars) ┘                                       └─ liefert das Spiel aus und
                                                            leitet die Züge weiter
```

Der Server speichert nichts. Er kennt nur die gerade laufenden Spiele und leitet die Spielzüge zwischen euch weiter.

## 1. Container Manager installieren

**Paket-Zentrum** → nach „Container Manager“ suchen → **Installieren**.

## 2. Galaxy Wars 2026 als Projekt anlegen

1. In **File Station** einen Ordner anlegen, zum Beispiel `docker/galaxy-wars`.
2. **Container Manager** → **Projekt** → **Erstellen**.
   - Projektname: `galaxy-wars`
   - Pfad: den Ordner aus Schritt 1 wählen
   - Quelle: **docker-compose.yml erstellen**, dann diesen Inhalt einfügen:

   ```yaml
   services:
     galaxy-wars:
       image: ghcr.io/pilotpirx73/galaxy-wars:latest
       container_name: galaxy-wars
       restart: unless-stopped
       ports:
         - "8090:8080"
   ```

3. **Weiter** → **Weiter** → **Fertig**. Der Container Manager lädt das Image herunter und startet den Container.

**Kurz testen:** Im Heimnetz `http://<IP-der-Synology>:8090` öffnen. Galaxy Wars 2026 sollte erscheinen.

> Ist Port 8090 auf deiner Synology schon belegt, ändere nur die linke Zahl, zum Beispiel `"8091:8080"`, und verwende diese Zahl auch in Schritt 3.

## 3. Reverse Proxy mit HTTPS einrichten

Damit das Spiel von außen verschlüsselt erreichbar ist:

**Systemsteuerung** → **Anmeldeportal** → **Erweitert** → **Reverse Proxy** → **Erstellen**

| Feld | Wert |
|---|---|
| Beschreibung | Galaxy Wars 2026 |
| **Quelle** – Protokoll | HTTPS |
| Quelle – Hostname | deine DynDNS-Adresse, z. B. `meinname.synology.me` |
| Quelle – Port | `8443` |
| **Ziel** – Protokoll | HTTP |
| Ziel – Hostname | `localhost` |
| Ziel – Port | `8090` |

Dann den Reiter **Benutzerdefinierter Header** öffnen → **Erstellen** → **WebSocket**. Damit werden zwei Einträge (`Upgrade` und `Connection`) automatisch hinzugefügt. **Ohne diesen Schritt funktioniert der Online-Modus nicht.** Mit **Speichern** abschließen.

**Zertifikat:** Wenn deine DynDNS-Adresse schon ein Let's-Encrypt-Zertifikat hat (Systemsteuerung → Sicherheit → Zertifikat), gilt es auch für Port 8443. Es muss nichts weiter eingerichtet werden.

## 4. Port im Router freigeben

Im Router (z. B. FRITZ!Box → Internet → Freigaben → Portfreigaben) eine Freigabe anlegen:

- **TCP-Port 8443** extern → **8443** an die Synology

Die DSM-Firewall muss Port 8443 ebenfalls zulassen, falls du sie aktiviert hast (Systemsteuerung → Sicherheit → Firewall).

## 5. Spielen

1. Öffne `https://meinname.synology.me:8443`.
2. Modus **„Online gegen Freund“** wählen, deinen Namen eingeben und auf **„Neues Online-Spiel erstellen“** klicken.
3. Den angezeigten **Code** oder **Link** an Lars schicken.
4. Lars öffnet den Link, gibt seinen Namen ein und klickt auf **„Beitreten“**. Los geht's!

Karte und Regeln legt fest, wer das Spiel erstellt. Bricht die Verbindung kurz ab, verbindet sich das Spiel automatisch wieder. Auch nach dem Neuladen der Seite geht es weiter.

**Test von außen:** `https://meinname.synology.me:8443/health` sollte `{"ok":true,…}` anzeigen. Am besten prüfst du das übers Handy mit ausgeschaltetem WLAN.

## Aktualisieren

Bei jeder Änderung am Spiel baut GitHub automatisch ein neues Image. So holst du es auf die Synology:

**Container Manager** → **Projekt** → `galaxy-wars` → **Aktion** → **Erstellen**. Dabei wird das neue Image geladen und der Container neu gestartet.

## Wenn es nicht klappt

| Problem | Lösung |
|---|---|
| Das Image lässt sich nicht herunterladen („denied“ / „unauthorized“) | Das Paket auf GitHub ist noch privat: GitHub → dein Profil → **Packages** → `galaxy-wars` → **Package settings** → **Change visibility** → **Public**. |
| Spiel lädt, aber „Kein Spielserver erreichbar“ | Im Reverse Proxy fehlt der **WebSocket**-Header (Schritt 3). |
| Von außen nicht erreichbar | Portfreigabe im Router und DSM-Firewall prüfen (Schritt 4). |
| „exec format error“ im Container-Protokoll | Sehr alte Synology mit 32-Bit-ARM-Prozessor werden nicht unterstützt. |
