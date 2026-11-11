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

## Zugang schützen und benachrichtigt werden

Ohne weitere Einstellungen kann jeder, der deine Adresse kennt, dort Online-Spiele eröffnen. Mit zwei Einträgen in der Projekt-Konfiguration änderst du das:

| Einstellung | Wirkung |
|---|---|
| `CREATE_PASSWORD` | Nur wer dieses Passwort kennt, kann ein **Online-Spiel erstellen**. Beitreten geht weiter nur mit dem Code bzw. Link des Erstellers, ganz ohne Passwort. Offline-Spielen (gegen den Computer, zu zweit an einem Gerät) bleibt frei. |
| `NOTIFY_URL` | Push-Nachricht aufs Handy, wenn jemand das Spiel aufruft (höchstens einmal pro halbe Stunde und Besucher), ein Online-Spiel erstellt, beitritt oder ein falsches Passwort probiert. |
| `NOTIFY_SHOW_IP` | `"true"` zeigt zusätzlich die IP-Adresse des Besuchers in der Nachricht. |
| `NOTIFY_SHOW_PLACE` | Zeigt den ungefähren Ort des Besuchers (z. B. „Oslo, Norwegen“), auch ohne diesen Eintrag. Dafür schickt der Server die IP-Adresse an den Dienst ipwho.is. `"false"` schaltet das ab. Der Ort ist nur grob, bei VPNs stimmt er nicht. |

Dazu gelten feste Obergrenzen: höchstens 20 Online-Spiele gleichzeitig, 8 Verbindungen pro Absender, und nach 5 falschen Passwörtern 10 Minuten Sperre.

### Benachrichtigungen mit ntfy einrichten

1. Die kostenlose App **ntfy** installieren (App Store bzw. Google Play), kein Konto nötig.
2. In der App auf **+** tippen und ein Thema abonnieren, z. B. `galaxy-wars-` gefolgt von einem langen, zufälligen Wort. Wer das Thema kennt, kann die Nachrichten mitlesen – es wirkt also wie ein Passwort.
3. Genau dieses Thema in `NOTIFY_URL` eintragen: `https://ntfy.sh/dein-thema`

Die Nachrichten laufen über den öffentlichen Dienst ntfy.sh. Sie enthalten Uhrzeit, Ereignis, Spielernamen und den ungefähren Ort (und die IP-Adresse nur, wenn du `NOTIFY_SHOW_IP` einschaltest).

### Einstellungen auf der Synology eintragen

1. **Container Manager** → **Projekt** → `galaxy-wars` auswählen → **Aktion** → **Stoppen**.
2. **Bearbeiten** (bzw. die YAML-Konfiguration des Projekts öffnen) und unter `ports:` diesen Block ergänzen – mit derselben Einrückung wie `ports:`:

   ```yaml
       environment:
         CREATE_PASSWORD: "dein-passwort"
         NOTIFY_URL: "https://ntfy.sh/dein-thema"
   ```

   ⚠️ **Kein `$` im Passwort** – der Container Manager liest `$…` als Variable und schneidet das Passwort dort ab. Am besten nur Buchstaben, Ziffern, `-` und `_` verwenden. Wer ein `$` braucht, schreibt es in der YAML-Datei doppelt (`$$`); im Spiel wird es normal eingegeben.

3. **Speichern** und das Projekt wieder **starten** bzw. **Erstellen**.
4. Prüfen: Im Protokoll des Containers steht `Passwort zum Erstellen: aktiv · Benachrichtigungen: aktiv`.

Die vollständige Beispiel-Datei liegt unter [`deploy/synology/docker-compose.yml`](../deploy/synology/docker-compose.yml).

## Aktualisieren

Bei jeder Änderung am Spiel baut GitHub automatisch ein neues Image.

**Von Hand:** **Container Manager** → **Projekt** → `galaxy-wars` → **Aktion** → **Erstellen**. Dabei wird das neue Image geladen und der Container neu gestartet.

**Automatisch jede Nacht:**

1. **Systemsteuerung** → **Aufgabenplaner** → **Erstellen** → **Geplante Aufgabe** → **Benutzerdefiniertes Script**
2. **Allgemein:** Aufgabe `Galaxy Wars aktualisieren`, Benutzer **root**
3. **Zeitplan:** täglich, zum Beispiel **4:00 Uhr**, also zu einer Zeit, in der niemand spielt. Ein Neustart beendet laufende Online-Spiele.
4. **Aufgabeneinstellungen → Benutzerdefiniertes Script:** den Inhalt von [`deploy/synology/update.sh`](../deploy/synology/update.sh) einfügen:

   ```sh
   PROJEKT=/volume1/docker/galaxy-wars   # Ordner des Container-Manager-Projekts

   export PATH="$PATH:/usr/local/bin"
   cd "$PROJEKT" || { echo "Ordner $PROJEKT nicht gefunden"; exit 1; }

   if docker compose version >/dev/null 2>&1; then DC="docker compose"; else DC="docker-compose"; fi

   $DC pull --quiet && $DC up -d
   docker image prune -f >/dev/null
   echo "Galaxy Wars: Update-Prüfung abgeschlossen ($(date))"
   ```

5. **OK**. Danach die Aufgabe einmal markieren und auf **Ausführen** klicken. Unter **Aktion → Ergebnis anzeigen** sollte „Update-Prüfung abgeschlossen“ stehen.

Liegt dein Projekt woanders, zum Beispiel auf `volume2`, passe die Zeile `PROJEKT=…` an. Den genauen Pfad zeigt die File Station: Rechtsklick auf den Ordner → **Eigenschaften**.

## Wenn es nicht klappt

| Problem | Lösung |
|---|---|
| Das Image lässt sich nicht herunterladen („denied“ / „unauthorized“) | Das Paket auf GitHub ist noch privat: GitHub → dein Profil → **Packages** → `galaxy-wars` → **Package settings** → **Change visibility** → **Public**. |
| Spiel lädt, aber „Kein Spielserver erreichbar“ | Im Reverse Proxy fehlt der **WebSocket**-Header (Schritt 3). |
| Von außen nicht erreichbar | Portfreigabe im Router und DSM-Firewall prüfen (Schritt 4). |
| „exec format error“ im Container-Protokoll | Sehr alte Synology mit 32-Bit-ARM-Prozessor werden nicht unterstützt. |
